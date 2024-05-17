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
    cli::{helpers::KeyVal, opts::Opts},
    runner::{
        wordlists::{compute_checksum, ParsedWordlist},
        Runner,
    },
    utils::{
        constants::{DEFAULT_FUZZ_KEY, DEFAULT_MODE, DEFAULT_STATUS_CODES},
        table::build_opts_table,
    },
};
use color_eyre::eyre::bail;
use color_eyre::eyre::Result;
use colored::Colorize;
use futures::{future::abortable, FutureExt};
use indicatif::HumanDuration;
use itertools::Itertools;
use log::{error, info, warn};
use merge::Merge;
use parking_lot::Mutex;
use ptree::print_tree;
use tokio::{io::AsyncWriteExt, task::JoinHandle, time::timeout};
use url::Url;
use utils::{structs::FuzzMatch, tree::UrlType};

use crate::utils::{
    constants::SUCCESS,
    structs::{Mode, Save},
    tree::{Tree, TreeData},
};

pub mod cli;
pub mod runner;
pub mod utils;

pub async fn _main(opts: Opts) -> Result<Tree<TreeData>> {
    if opts.url.is_none() && !opts.resume {
        bail!("Missing URL");
    }
    if opts.wordlists.is_empty() && !opts.resume {
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

    // Merge saved options with the ones passed as arguments
    let mut opts = if let Some(ref save) = saved_json {
        if let Ok(mut saved_opts) = save.as_ref().map(|x| x.opts.clone()) {
            saved_opts.merge(opts.clone());
            saved_opts
        } else {
            let err = save.as_ref().unwrap_err();
            error!(
                "Error while parsing save file: {}, using passed options",
                err.to_string().bold().red()
            );
            opts.clone()
        }
    } else {
        opts.clone()
    };

    // Default status filters
    if !opts.filter.iter().any(|e| e.0 == "status") {
        let mut filters = opts.filter.clone();
        filters.push(KeyVal(
            "status".to_string(),
            DEFAULT_STATUS_CODES.to_string(),
        ));
        opts.filter = filters;
    }

    // Parse wordlists into a HashMap associating each wordlist key to its contents
    let mut words = runner::wordlists::parse(&opts.wordlists).await?;

    // Get the number of threads to use, default to 10 times the number of cores
    let threads = opts
        .threads
        .unwrap_or(num_cpus::get() * 10)
        .max(1)
        .min(words.iter().fold(0, |acc, (_, v)| acc + v.words.len()));

    let mut url = opts.url.clone().unwrap();

    // Check if the URL contains any of the replace keywords
    let mut fuzz_matches = words
        .keys()
        .flat_map(|x| {
            url.match_indices(x).map(|(i, e)| FuzzMatch {
                content: e.to_string(),
                start: i,
                end: i + e.len(),
            })
        })
        .collect::<Vec<_>>();
    // Set the mode based on the options and the URL
    let mode: Mode = if opts.mode.is_some() {
        opts.mode.as_deref().unwrap().into()
    } else if opts.depth.is_some() {
        Mode::Recursive
    } else if !fuzz_matches.is_empty() {
        Mode::Classic
    } else {
        DEFAULT_MODE.into()
    };

    match mode {
        Mode::Recursive => {
            if !fuzz_matches.is_empty() {
                warn!(
                    "URL contains the replace keyword{}: {}, this is supported with {}",
                    if fuzz_matches.len() > 1 { "s" } else { "" },
                    fuzz_matches
                        .iter()
                        .map(|e| e.content.clone())
                        .join(", ")
                        .bold()
                        .blue(),
                    format!("{} {}", "--mode".dimmed(), "classic".bold())
                );
            }
        }
        Mode::Classic => {
            if fuzz_matches.is_empty() {
                url = url.trim_end_matches('/').to_string() + "/" + DEFAULT_FUZZ_KEY;
                fuzz_matches.push(FuzzMatch {
                    content: DEFAULT_FUZZ_KEY.to_string(),
                    start: url.len() - 1,
                    end: url.len() - 1 + DEFAULT_FUZZ_KEY.len(),
                });
                warn!(
                    "URL does not contain the replace keyword: {}, it will be treated as: {}",
                    DEFAULT_FUZZ_KEY.bold(),
                    url.bold()
                );
            }
            // Remove unused wordlists keys
            for k in words.keys().cloned().collect::<Vec<_>>() {
                if !fuzz_matches.iter().any(|e| e.content == k) {
                    warn!(
                        "Wordlist {} is not used in the URL, removing it",
                        k.bold().blue()
                    );
                    words.remove(&k);
                }
            }
        }
        Mode::Spider => {
            if !fuzz_matches.is_empty() {
                warn!(
                    "URL contains the replace keyword{}: {}, this is supported with {}",
                    if fuzz_matches.len() > 1 { "s" } else { "" },
                    fuzz_matches
                        .iter()
                        .map(|e| e.content.clone())
                        .join(", ")
                        .bold()
                        .blue(),
                    format!("{} {}", "--mode".dimmed(), "classic".bold())
                );
            }
        }
    }

    let before = words.values().fold(0, |acc, x| acc + x.words.len());

    // Apply filters and transformations to the wordlists (if any)
    runner::wordlists::filters(&opts, &mut words)?;
    runner::wordlists::transformations(&opts, &mut words);

    runner::wordlists::deduplicate(&mut words);

    if !opts.quiet {
        println!(
            "{}",
            build_opts_table(&opts, &words, &mode, threads, url.clone(), &fuzz_matches)
        );
    }

    let after = words.values().fold(0, |acc, x| acc + x.words.len());
    if before != after && !opts.quiet {
        info!(
            "{} words loaded, {} after deduplication and filters (-{}%)",
            before.to_string().bold().blue(),
            after.to_string().bold().blue(),
            ((before - after) as f64 / before as f64 * 100.0)
                .round()
                .to_string()
                .bold()
                .green()
        );
    }

    if words.values().all(|x| x.words.is_empty()) {
        bail!("No words found in wordlists");
    }

    // These will be used to keep track of the current state of the tree across threads
    let current_depth = Arc::new(Mutex::new(0));
    let current_indexes: Arc<Mutex<HashMap<String, Vec<usize>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let saved_tree = if opts.resume {
        match saved_json {
            Some(json) if json.is_ok() => Some(utils::tree::from_save(
                &opts,
                &json.unwrap(),
                current_depth.clone(),
                current_indexes.clone(),
                words.clone(),
            )?),
            _ => None,
        }
    } else {
        None
    };
    // We need to define this here for later use
    let has_saved = saved_tree.is_some();

    // Create the tree
    let tree = if let Some(saved_tree) = saved_tree {
        // Resume from the saved state
        saved_tree
    } else {
        // Create the tree with the root URL
        let t = Arc::new(Mutex::new(Tree::new()));
        let cleaned_url = match mode {
            Mode::Recursive | Mode::Spider => url.clone(),
            Mode::Classic => {
                // Get the first part of the url, before the first occurence of a fuzz key from fuzz_matches
                let mut smallest_index = url.len();
                for match_ in &fuzz_matches {
                    if let Some(index) = url.find(&match_.content) {
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
                url_type: UrlType::Directory,
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
        // Exit if the root URL is down and the user didn't specify to force the execution
        if !opts.force {
            bail!("Root URL is down, use --force to continue");
        }
    } else {
        tree.lock().root.clone().unwrap().lock().data.status_code = res?.status().as_u16();
    }

    let start_time = std::time::Instant::now();

    if !opts.quiet {
        info!(
            "Press {} to {}exit",
            "Ctrl+C".bold(),
            if opts.no_save { "" } else { "save state and " }
        );
    }

    // Define the main function to run based on the mode
    let main_fun = match mode {
        Mode::Recursive => runner::recursive::Recursive::new(
            opts.clone(),
            current_depth.clone(),
            tree.clone(),
            current_indexes.clone(),
            // Split the words into chunks of equal size for each thread
            Arc::new(
                words
                    .iter()
                    .fold(
                        Vec::new(),
                        |mut acc, (_, ParsedWordlist { words: w, .. })| {
                            acc.extend(w.clone());
                            acc
                        },
                    )
                    .chunks(words.iter().fold(0, |acc, (_, v)| acc + v.words.len()) / threads)
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
            // We do not need to chunk the words here
            words.clone(),
            threads,
        )
        .run()
        .boxed(),
        Mode::Spider => {
            runner::spider::Spider::new(url.clone(), opts.clone(), tree.clone(), threads)
                .run()
                .boxed()
        }
    };
    // Run the main function with a timeout if specified
    let (task, handle) = if let Some(max_time) = opts.max_time {
        abortable(timeout(Duration::from_secs(max_time as u64), main_fun).into_inner())
    } else {
        abortable(main_fun)
    };

    let main_thread = tokio::spawn(task);
    let aborted = Arc::new(AtomicBool::new(false));
    // Create a channel to receive the abort signal
    // TODO: Maybe we could use a oneshot channel here
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);

    // We need to clone the variables to be used in the signal handler
    // TODO: Find a better way to do this
    let ctrlc_tree = tree.clone();
    let ctrlc_depth = current_depth.clone();
    let ctrlc_words = words.clone();
    let ctrlc_opts = opts.clone();
    let ctrlc_aborted = aborted.clone();
    let ctrlc_save_file = opts.save_file.clone();

    let (ctrlc_task, ctrlc_handle) = abortable(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen to Ctrl-C");

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
                let mut file = tokio::fs::File::create(
                    &ctrlc_save_file
                        .clone()
                        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No save file"))?,
                )
                .await?;

                file.write_all(content.as_bytes()).await?;
                file.flush().await?;
                print!("\x1B[2K\r");
                info!(
                    "Saved state to {}",
                    ctrlc_save_file
                        .clone()
                        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No save file"),)?
                        .to_string()
                        .bold()
                );
            }
        }
        tx.send(()).await?;
        Ok(())
    });

    let signals_task: JoinHandle<Result<Result<()>, futures::future::Aborted>> =
        tokio::spawn(ctrlc_task);
    let abort_res = main_thread.await?;

    let thread_res = match abort_res {
        Ok(res) => Some(res),
        Err(_) => None,
    };

    if thread_res.is_some() {
        if let Err(e) = thread_res.unwrap() {
            error!("{}", e);
        } else {
            if !opts.quiet {
                println!(
                    "{} Done in {} with an average of {} req/s",
                    SUCCESS.to_string().green(),
                    HumanDuration(std::time::Instant::now().duration_since(start_time))
                        .to_string()
                        .bold(),
                    ((match mode {
                        Mode::Recursive =>
                            words.iter().fold(0, |acc, (_, v)| acc + v.words.len())
                                * *current_depth.lock(),
                        Mode::Classic => {
                            words.iter().fold(0, |acc, (_, v)| acc + v.words.len())
                        }
                        Mode::Spider => 1,
                    }) as f64
                        / std::time::Instant::now()
                            .duration_since(start_time)
                            .as_secs_f64())
                    .round()
                    .to_string()
                    .bold()
                );
            }

            let root = tree.lock().root.clone().unwrap().clone();

            if !opts.quiet {
                print_tree(&*root.lock())?;
            }

            // Remove save file after finishing resuming
            if has_saved && !opts.keep_save {
                tokio::fs::remove_file(opts.save_file.clone().unwrap()).await?;
            }
            if opts.output.is_some() {
                let res = utils::save_to_file(&opts, root, current_depth, tree.clone());

                match res {
                    Ok(_) => info!("Saved to {}", opts.output.unwrap().bold()),
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        }
    }

    // Wait for the abort signal if aborted has been set
    if aborted.load(Ordering::Relaxed) {
        rx.recv().await;
    }

    // Terminate the signal stream.
    ctrlc_handle.abort();

    // Wait for the signal handler to finish
    let signals_res = signals_task.await?;
    // If we didn't abort and the signal handler returns an error, this is unexpected
    // Because the signal handler should only error when aborted
    if aborted.load(Ordering::Relaxed) {
        if let Err(e) = signals_res {
            error!("{}", e);
        }
    }
    let tree = tree.lock().clone();
    Ok(tree)
}
