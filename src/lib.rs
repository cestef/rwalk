use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::cli::opts::Opts;
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
    let mode: Mode = if opts.depth.unwrap_or(1) > 1 {
        Mode::Recursive
    } else {
        opts.mode.as_deref().unwrap_or("recursive").into()
    };
    let mut url = opts.url.clone().unwrap();
    match mode {
        Mode::Recursive => {
            if url
                .matches(opts.fuzz_key.clone().unwrap_or("$".to_string()).as_str())
                .count()
                > 0
            {
                warn!(
                    "URL contains the replace keyword: {}, this is supported with {}",
                    opts.fuzz_key.clone().unwrap().bold(),
                    format!(
                        "{} {} | {}",
                        "--mode".dimmed(),
                        "permutations".bold(),
                        "classic".bold()
                    )
                );
            }
        }
        Mode::Classic => {
            if url.matches(opts.fuzz_key.clone().unwrap().as_str()).count() == 0 {
                url = url.trim_end_matches('/').to_string()
                    + "/"
                    + opts.fuzz_key.clone().unwrap().as_str();
                warn!(
                    "URL does not contain the replace keyword: {}, it will be treated as: {}",
                    opts.fuzz_key.clone().unwrap().bold(),
                    url.bold()
                );
            }
        }
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

    let before = words.len();

    runner::wordlists::filters(&opts, &mut words)?;
    runner::wordlists::transformations(&opts, &mut words);

    words.sort_unstable();
    words.dedup();

    let after = words.len();
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
    if words.is_empty() {
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
            Mode::Classic => url
                .split(opts.fuzz_key.clone().unwrap().as_str())
                .collect::<Vec<_>>()[0]
                .to_string(),
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
        .min(words.len());
    info!(
        "Starting crawler with {} threads",
        threads.to_string().bold(),
    );

    let watch = stopwatch::Stopwatch::start_new();

    info!(
        "Press {} to {}exit",
        "Ctrl+C".bold(),
        if opts.no_save { "" } else { "save state and " }
    );
    let main_fun = match mode {
        Mode::Recursive => runner::recursive::run(
            opts.clone(),
            current_depth.clone(),
            tree.clone(),
            current_indexes.clone(),
            Arc::new(
                words
                    .chunks(words.len() / threads)
                    .map(|x| x.to_vec())
                    .collect::<Vec<_>>(),
            ),
            words.clone(),
        )
        .boxed(),
        Mode::Classic => runner::classic::run(
            url.clone(),
            opts.clone(),
            tree.clone(),
            words.clone(),
            threads,
        )
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
                        let checksum = format!("{:x}", md5::compute(ctrlc_words.join("\n")));
                        let content = serde_json::to_string(&Save {
                            tree: ctrlc_tree.clone(),
                            depth: ctrlc_depth.clone(),
                            wordlist_checksum: checksum,
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
    if let Ok(_) = res {
        println!(
            "{} Done in {} with an average of {} req/s",
            SUCCESS.to_string().green(),
            HumanDuration(watch.elapsed()).to_string().bold(),
            ((match mode {
                Mode::Recursive => words.len() * *current_depth.lock(),
                Mode::Classic =>
                    if opts.permutations {
                        words.len().pow(url.matches(opts.fuzz_key.clone().unwrap().as_str()).count() as u32)
                    } else {
                        words.len()
                    },
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
