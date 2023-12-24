#![allow(dead_code)]

use crate::{
    constants::{SAVE_FILE, SUCCESS},
    tree::{Tree, TreeData},
    utils::{apply_filters, apply_transformations, parse_wordlists, save_to_file},
};
use anyhow::Result;
use clap::Parser;
use cli::Opts;
use colored::Colorize;
use futures::future::abortable;
use futures::stream::StreamExt;
use indicatif::HumanDuration;
use log::{error, info, warn};
use logger::init_logger;
use parking_lot::Mutex;
use ptree::print_tree;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::timeout;

use url::Url;

mod cli;
mod constants;
mod core;
mod logger;
mod tree;
mod utils;

#[derive(Serialize, Deserialize)]
struct Save {
    tree: Arc<Mutex<Tree<TreeData>>>,
    depth: Arc<Mutex<usize>>,
    wordlist_checksum: String,
    indexes: HashMap<String, Vec<usize>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    let config_path = dirs::home_dir()
        .unwrap()
        .join(".config")
        .join("rwalk")
        .join(".env");
    // println!("{}", config_path.to_str().unwrap());
    dotenv::from_path(config_path).ok();
    let opts = Opts::parse();

    if opts.no_color {
        colored::control::set_override(false);
    }
    if !opts.quiet {
        utils::banner();
    }
    if opts.interactive {
        cli::main_interactive().await
    } else {
        _main(opts.clone()).await
    }
}

pub async fn _main(opts: Opts) -> Result<()> {
    if opts.url.is_none() {
        error!("Missing URL");
        return Ok(());
    }
    let mut words = if opts.wordlists.len() == 1 && opts.wordlists.first().unwrap() == "-" {
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();

        let mut buf = String::new();
        handle.read_to_string(&mut buf)?;

        let words: Vec<String> = buf
            .split('\n')
            .map(|x| x.to_string())
            .filter(|x| !x.is_empty())
            .collect();

        if words.is_empty() {
            error!("Missing wordlists");
            return Ok(());
        }
        words
    } else {
        parse_wordlists(&opts.wordlists)?
    };

    let before = words.len();
    apply_filters(&opts, &mut words)?;
    apply_transformations(&opts, &mut words);
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
    if words.len() == 0 {
        error!("No words found in wordlists");
        return Ok(());
    }
    let depth = Arc::new(Mutex::new(0));
    let current_indexes: Arc<Mutex<HashMap<String, Vec<usize>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let saved = std::fs::read_to_string(opts.save_file.clone());
    let saved = if opts.resume {
        match saved {
            Ok(saved) => {
                let saved: Save = serde_json::from_str(&saved)?;
                if let Some(root) = &saved.tree.clone().lock().root {
                    if root.lock().data.url != opts.url.clone().unwrap() {
                        None
                    } else {
                        print_tree(&*root.lock())?;
                        info!(
                            "Found saved state crawled to depth {}",
                            (*saved.depth.lock() + 1).to_string().blue()
                        );

                        *depth.lock() = *saved.depth.lock();
                        if saved.wordlist_checksum == {
                            let mut hasher = Sha256::new();
                            hasher.update(words.join("\n"));
                            format!("{:x}", hasher.finalize())
                        } {
                            *current_indexes.lock() = saved.indexes;
                        } else {
                            warn!(
                                "Wordlists have changed, starting from scratch at depth {}",
                                (*saved.depth.lock() + 1).to_string().yellow()
                            );
                        }
                        Some(saved.tree)
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };
    let has_saved = saved.is_some();
    let tree = if let Some(saved) = saved {
        saved
    } else {
        let t = Arc::new(Mutex::new(Tree::new()));
        t.lock().insert(
            TreeData {
                url: opts.url.clone().unwrap(),
                depth: 0,
                path: Url::parse(&opts.url.clone().unwrap())?
                    .path()
                    .to_string()
                    .trim_end_matches('/')
                    .to_string(),
                status_code: 0,
            },
            None,
        );
        t
    };

    // Check if the root URL is up
    let root_url = tree.lock().root.clone().unwrap().lock().data.url.clone();
    let root_url = Url::parse(&root_url)?;
    let tmp_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(opts.timeout.unwrap() as u64))
        .connect_timeout(Duration::from_secs(opts.timeout.unwrap() as u64))
        .build()?;
    let res = tmp_client.get(root_url.clone()).send().await;
    if let Err(e) = res {
        error!("Error while connecting to {}: {}", root_url, e);
        return Ok(());
    }
    let res = res.unwrap();

    tree.lock().root.clone().unwrap().lock().data.status_code = res.status().as_u16();

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

    info!("Press {} to save state and exit", "Ctrl+C".bold());
    let chunks = words
        .chunks(words.len() / threads)
        .map(|x| x.to_vec())
        .collect::<Vec<_>>();
    let chunks = Arc::new(chunks);

    let main_fun = core::start(
        opts.clone(),
        depth.clone(),
        tree.clone(),
        current_indexes.clone(),
        chunks.clone(),
        words.clone(),
    );
    let (task, handle) = if let Some(max_time) = opts.max_time {
        abortable(timeout(Duration::from_secs(max_time as u64), main_fun).into_inner())
    } else {
        abortable(main_fun)
    };
    let main_thread = tokio::spawn(task);
    let aborted = Arc::new(AtomicBool::new(false));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);

    let ctrlc_tree = tree.clone();
    let ctrlc_depth = depth.clone();
    let ctrlc_words = words.clone();
    let ctrlc_aborted = aborted.clone();
    let ctrlc_save_file = opts.save_file.clone();
    let mut signals = Signals::new(&[SIGINT])?;
    let ctrlc_handle = signals.handle();

    let signals_task = tokio::spawn(async move {
        while let Some(signal) = signals.next().await {
            match signal {
                SIGINT => {
                    info!("Aborting...");
                    ctrlc_aborted.store(true, Ordering::Relaxed);
                    handle.abort();
                    if !opts.no_save {
                        let checksum = {
                            let mut hasher = Sha256::new();
                            hasher.update(ctrlc_words.join("\n"));
                            hasher.finalize()
                        };
                        let checksum = format!("{:x}", checksum);
                        let content = serde_json::to_string(&Save {
                            tree: ctrlc_tree.clone(),
                            depth: ctrlc_depth.clone(),
                            wordlist_checksum: checksum,
                            indexes: current_indexes.lock().clone(),
                        });
                        match content {
                            Ok(content) => {
                                let mut file = std::fs::File::create(&ctrlc_save_file).unwrap();
                                file.write_all(content.as_bytes()).unwrap();
                                file.flush().unwrap();
                                print!("\x1B[2K\r");
                                info!("Saved state to {}", ctrlc_save_file.bold());
                            }
                            Err(_) => {}
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
            ((words.len() * *depth.lock()) as f64 / watch.elapsed().as_secs_f64())
                .round()
                .to_string()
                .bold()
        );

        let root = tree.lock().root.clone().unwrap().clone();

        print_tree(&*root.lock())?;

        // Remove save file if it's the default one
        if has_saved && opts.save_file == SAVE_FILE {
            std::fs::remove_file(opts.save_file.clone())?;
        }
        if opts.output.is_some() {
            let res = save_to_file(&opts, root, depth, tree);

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
