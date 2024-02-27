use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{
    cli::opts::Opts,
    runner::{wordlists::compute_checksum, Runner},
    utils::constants::{DEFAULT_FUZZ_KEY, DEFAULT_MODE},
};
use anyhow::bail;
use anyhow::Result;
use colored::Colorize;
use futures::{future::abortable, FutureExt, StreamExt};
use indicatif::HumanDuration;
use log::{error, info, warn};
use merge::Merge;
use parking_lot::Mutex;
use ptree::print_tree;
use signal_hook::consts::SIGINT;
use signal_hook_tokio::Signals;
use tokio::{io::AsyncWriteExt, time::timeout};
use url::Url;

use crate::utils::{
    constants::SUCCESS,
    structs::{Mode, Save},
    tree::{Tree, TreeData},
};

pub mod cli;
pub mod runner;
pub mod utils;

pub async fn _main(opts: Opts) -> Result<()> {
    if opts.url.is_none() && !opts.resume {
        bail!("Missing URL");
    }
    if opts.wordlists.is_empty() {
        bail!("Missing wordlists");
    }

    let saved = if opts.resume {
        let res = tokio::fs::read_to_string(opts.save_file.clone().unwrap()).await;
        if res.is_err() {
            bail!("No save file: {}", opts.save_file.clone().unwrap().dimmed());
        }
        res
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "No save file"))
    };
    let saved_json = if let Ok(saved) = saved {
        Some(serde_json::from_str::<Save>(&saved))
    } else {
        None
    };

    let opts = if let Some(ref save) = saved_json {
        let mut saved_opts = save.as_ref().unwrap().opts.clone();
        saved_opts.merge(opts.clone());
        saved_opts
    } else {
        opts.clone()
    };

    let mut words = runner::wordlists::parse(&opts.wordlists).await?;

    let mut url = opts.url.clone().unwrap();
    let fuzz_matches = words
        .keys()
        .filter(|x| url.contains(*x))
        .cloned()
        .collect::<Vec<_>>();
    let mode: Mode = if opts.mode.is_some() {
        opts.mode.as_deref().unwrap().into()
    } else if opts.depth.is_some() {
        Mode::Recursive
    } else if !fuzz_matches.is_empty() {
        Mode::Classic
    } else {
        DEFAULT_MODE.into()
    };
    info!("Mode: {}", mode.to_string().bold());
    match mode {
        Mode::Recursive => {
            if !fuzz_matches.is_empty() {
                warn!(
                    "URL contains the replace keyword{}: {}, this is supported with {}",
                    if fuzz_matches.len() > 1 { "s" } else { "" },
                    fuzz_matches.join(", ").bold(),
                    format!("{} {}", "--mode".dimmed(), "classic".bold())
                );
            }
        }
        Mode::Classic => {
            if fuzz_matches.is_empty() {
                url = url.trim_end_matches('/').to_string() + "/" + DEFAULT_FUZZ_KEY;
                warn!(
                    "URL does not contain the replace keyword: {}, it will be treated as: {}",
                    DEFAULT_FUZZ_KEY.bold(),
                    url.bold()
                );
            }
        }
    }
    let before = words.values().fold(0, |acc, x| acc + x.len());

    runner::wordlists::filters(&opts, &mut words)?;
    runner::wordlists::transformations(&opts, &mut words);

    runner::wordlists::deduplicate(&mut words);

    let after = words.values().fold(0, |acc, x| acc + x.len());
    if before != after {
        info!(
            "{} words loaded, {} after deduplication and filters (-{}%)",
            before.to_string().bold(),
            after.to_string().bold(),
            ((before - after) as f64 / before as f64 * 100.0)
                .round()
                .to_string()
                .bold()
        );
    } else {
        info!("{} words loaded", before.to_string().bold());
    }

    if words.values().all(|x| x.is_empty()) {
        bail!("No words found in wordlists");
    }
    let current_depth = Arc::new(Mutex::new(0));
    let current_indexes: Arc<Mutex<HashMap<String, Vec<usize>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let saved_tree = if opts.resume {
        match saved_json {
            Some(json) => Some(utils::tree::from_save(
                &opts,
                &json.unwrap(),
                current_depth.clone(),
                current_indexes.clone(),
                words.clone(),
            )?),
            None => None,
        }
    } else {
        None
    };

    let has_saved = saved_tree.is_some();

    let tree = if let Some(saved_tree) = saved_tree {
        saved_tree
    } else {
        let t = Arc::new(Mutex::new(Tree::new()));
        let cleaned_url = match mode {
            Mode::Recursive => url.clone(),
            Mode::Classic => {
                // Get the first part of the url, before the first occurence of a fuzz key from fuzz_matches
                let mut smallest_index = url.len();
                for match_ in &fuzz_matches {
                    if let Some(index) = url.find(match_) {
                        if index < smallest_index {
                            smallest_index = index;
                        }
                    }
                }
                url[..smallest_index].to_string()
            }
        };
        t.lock().insert(
            TreeData {
                url: cleaned_url.clone(),
                depth: 0,
                path: Url::parse(&cleaned_url.clone())?
                    .path()
                    .to_string()
                    .trim_end_matches('/')
                    .to_string(),
                status_code: 0,
                extra: serde_json::Value::Null,
            },
            None,
        );
        t
    };

    // Check if the root URL is up
    let root_url = tree.lock().root.clone().unwrap().lock().data.url.clone();
    let root_url = Url::parse(&root_url)?;

    let tmp_client = runner::client::build(&opts)?;

    let res = tmp_client.get(root_url.clone()).send().await;
    if let Err(e) = res {
        error!("Error while connecting to {}: {}", root_url, e);
        if !opts.force {
            return Ok(());
        }
    } else {
        tree.lock().root.clone().unwrap().lock().data.status_code = res?.status().as_u16();
    }

    let threads = opts
        .threads
        .unwrap_or(num_cpus::get() * 10)
        .max(1)
        .min(words.iter().fold(0, |acc, (_, v)| acc + v.len()));
    info!(
        "Starting crawler with {} thread{}",
        threads.to_string().bold(),
        if threads > 1 { "s" } else { "" }
    );

    let watch = stopwatch::Stopwatch::start_new();

    info!(
        "Press {} to {}exit",
        "Ctrl+C".bold(),
        if opts.no_save { "" } else { "save state and " }
    );

    let main_fun = match mode {
        Mode::Recursive => runner::recursive::Recursive::new(
            opts.clone(),
            current_depth.clone(),
            tree.clone(),
            current_indexes.clone(),
            Arc::new(
                words
                    .iter()
                    .fold(Vec::new(), |mut acc, (_, v)| {
                        acc.extend(v.clone());
                        acc
                    })
                    .chunks(words.iter().fold(0, |acc, (_, v)| acc + v.len()) / threads)
                    .map(|x| x.to_vec())
                    .collect::<Vec<_>>(),
            ),
        )
        .run()
        .boxed(),
        Mode::Classic => runner::classic::Classic::new(
            url.clone(),
            opts.clone(),
            tree.clone(),
            words.clone(),
            threads,
        )
        .run()
        .boxed(),
    };
    let (task, handle) = if let Some(max_time) = opts.max_time {
        abortable(timeout(Duration::from_secs(max_time as u64), main_fun).into_inner())
    } else {
        abortable(main_fun)
    };
    let main_thread = tokio::spawn(task);
    let aborted = Arc::new(AtomicBool::new(false));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);

    let ctrlc_tree = tree.clone();
    let ctrlc_depth = current_depth.clone();
    let ctrlc_words = words.clone();
    let ctrlc_opts = opts.clone();
    let ctrlc_aborted = aborted.clone();
    let ctrlc_save_file = opts.save_file.clone();
    let mut signals = Signals::new([SIGINT])?;
    let ctrlc_handle = signals.handle();

    let signals_task = tokio::spawn(async move {
        while let Some(signal) = signals.next().await {
            match signal {
                SIGINT => {
                    info!("Aborting...");
                    ctrlc_aborted.store(true, Ordering::Relaxed);
                    handle.abort();
                    if !opts.no_save {
                        let content = serde_json::to_string(&Save {
                            tree: ctrlc_tree.clone(),
                            depth: ctrlc_depth.clone(),
                            wordlist_checksum: compute_checksum(&ctrlc_words),
                            indexes: current_indexes.lock().clone(),
                            opts: ctrlc_opts.clone(),
                        });
                        if let Ok(content) = content {
                            let mut file =
                                tokio::fs::File::create(&ctrlc_save_file.clone().unwrap())
                                    .await
                                    .unwrap();
                            file.write_all(content.as_bytes()).await.unwrap();
                            file.flush().await.unwrap();
                            print!("\x1B[2K\r");
                            info!("Saved state to {}", ctrlc_save_file.clone().unwrap().bold());
                        }
                    }
                    tx.send(()).await.unwrap();
                }
                _ => unreachable!(),
            }
        }
    });
    let res = main_thread.await?;
    if res.is_ok() {
        println!(
            "{} Done in {} with an average of {} req/s",
            SUCCESS.to_string().green(),
            HumanDuration(watch.elapsed()).to_string().bold(),
            ((match mode {
                Mode::Recursive =>
                    words.iter().fold(0, |acc, (_, v)| acc + v.len()) * *current_depth.lock(),
                Mode::Classic => {
                    words.iter().fold(0, |acc, (_, v)| acc + v.len())
                }
            }) as f64
                / watch.elapsed().as_secs_f64())
            .round()
            .to_string()
            .bold()
        );

        let root = tree.lock().root.clone().unwrap().clone();

        if !opts.quiet {
            print_tree(&*root.lock())?;
        }

        // Remove save file after finishing resuming
        if has_saved && !opts.keep_save {
            tokio::fs::remove_file(opts.save_file.clone().unwrap()).await?;
        }
        if opts.output.is_some() {
            let res = utils::save_to_file(&opts, root, current_depth, tree);

            match res {
                Ok(_) => info!("Saved to {}", opts.output.unwrap().bold()),
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
    }
    if aborted.load(Ordering::Relaxed) {
        rx.recv().await;
    }

    // Terminate the signal stream.
    ctrlc_handle.close();
    signals_task.await?;
    Ok(())
}
