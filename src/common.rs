use colored::Colorize;
use serde_json::json;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use parking_lot::Mutex;

use crate::{
    cli::Opts,
    constants::{ERROR, PROGRESS_CHARS, PROGRESS_TEMPLATE, SUCCESS, WARNING},
    tree::{Tree, TreeData},
};

pub async fn start(
    opts: Opts,
    depth: Arc<Mutex<usize>>,
    tree: Arc<Mutex<Tree<TreeData>>>,
    current_indexes: Arc<Mutex<HashMap<String, Vec<usize>>>>,
    chunks: Arc<Vec<Vec<String>>>,
    words: Vec<String>,
) -> Result<()> {
    while *depth.lock() < opts.depth.unwrap() {
        let previous_nodes = tree.lock().get_nodes_at_depth(depth.lock().clone());
        let root_progress = indicatif::MultiProgress::new();
        let mut progresses = HashMap::new();
        let mut handles = Vec::new();
        for previous_node in &previous_nodes {
            let mut indexes = current_indexes.lock();
            let index = indexes
                .entry(previous_node.lock().data.url.clone())
                .or_insert_with(|| vec![0; chunks.len()]);

            let pb = root_progress
                .add(indicatif::ProgressBar::new((words.len()) as u64))
                .with_style(
                    indicatif::ProgressStyle::default_bar()
                        .template(PROGRESS_TEMPLATE)?
                        .progress_chars(PROGRESS_CHARS),
                )
                .with_message(format!(
                    "/{}",
                    previous_node.lock().data.path.trim_start_matches("/")
                ))
                .with_prefix(format!("d={}", *depth.lock() + 1))
                .with_position(index.iter().sum::<usize>() as u64);
            pb.enable_steady_tick(Duration::from_millis(100));
            progresses.insert(previous_node.lock().data.url.clone(), pb);

            let progress = progresses.get(&previous_node.lock().data.url).unwrap();

            let client = crate::client::build(&opts)?;

            for (i, chunk) in chunks.iter().enumerate() {
                let mut tree = tree.lock().clone();
                let previous_node = previous_node.clone();
                let chunk = chunk.clone();
                let client = client.clone();
                let progress = progress.clone();
                let indexes = current_indexes.clone();
                let opts = opts.clone();
                let handle = tokio::spawn(async move {
                    while indexes
                        .lock()
                        .get_mut(&previous_node.lock().data.url)
                        .unwrap()[i]
                        < chunk.len()
                    {
                        let index = indexes
                            .lock()
                            .get_mut(&previous_node.lock().data.url)
                            .unwrap()[i];
                        let word = chunk[index].clone();
                        let data = previous_node.lock().data.clone();

                        let mut url = data.url.clone();
                        match url.ends_with('/') {
                            true => url.push_str(&word),
                            false => url.push_str(&format!("/{}", word)),
                        }

                        let sender = crate::client::get_sender(&opts, &url, &client);

                        let t1 = Instant::now();

                        let response = sender.send().await;

                        if let Some(throttle) = opts.throttle {
                            if throttle > 0 {
                                let elapsed = t1.elapsed();
                                let sleep_duration = Duration::from_secs_f64(1.0 / throttle as f64);
                                if let Some(sleep) = sleep_duration.checked_sub(elapsed) {
                                    tokio::time::sleep(sleep).await;
                                }
                            }
                        }
                        match response {
                            Ok(mut response) => {
                                let status_code = response.status().as_u16();
                                let mut text = String::new();
                                while let Ok(chunk) = response.chunk().await {
                                    if let Some(chunk) = chunk {
                                        text.push_str(&String::from_utf8_lossy(&chunk));
                                    } else {
                                        break;
                                    }
                                }
                                let filtered = crate::filters::check(
                                    &opts,
                                    &text,
                                    status_code,
                                    t1.elapsed().as_millis(),
                                );

                                if filtered {
                                    let additions =
                                        crate::filters::parse_show(&opts, &text, &response);

                                    progress.println(format!(
                                        "{} {} {} {}{}",
                                        if response.status().is_success() {
                                            SUCCESS.to_string().green()
                                        } else if response.status().is_redirection() {
                                            WARNING.to_string().yellow()
                                        } else {
                                            ERROR.to_string().red()
                                        },
                                        response.status().as_str().bold(),
                                        url,
                                        format!(
                                            "{}ms",
                                            t1.elapsed().as_millis().to_string().bold()
                                        )
                                        .dimmed(),
                                        additions.iter().fold("".to_string(), |acc, addition| {
                                            format!(
                                                "{} | {}: {}",
                                                acc,
                                                addition.key.dimmed().bold(),
                                                addition.value.dimmed()
                                            )
                                        })
                                    ));
                                    // Check if this path is already in the tree
                                    let found = previous_node
                                        .lock()
                                        .children
                                        .iter()
                                        .any(|child| child.lock().data.path == *word);

                                    if !found {
                                        tree.insert(
                                            TreeData {
                                                url: url.clone(),
                                                depth: data.depth + 1,
                                                path: word.clone(),
                                                status_code,
                                                extra: json!(additions),
                                            },
                                            Some(previous_node.clone()),
                                        );
                                    } else {
                                        progress.println(format!(
                                            "{} {} {}",
                                            WARNING.to_string().yellow(),
                                            "Already in tree".bold(),
                                            url
                                        ));
                                    }
                                }
                            }
                            Err(err) => {
                                if err.is_timeout() {
                                    progress.println(format!(
                                        "{} {} {}",
                                        ERROR.to_string().red(),
                                        "Timeout reached".bold(),
                                        url
                                    ));
                                } else if err.is_redirect() {
                                    progress.println(format!(
                                        "{} {} {} {}",
                                        WARNING.to_string().yellow(),
                                        "Redirect limit reached".bold(),
                                        url,
                                        "Check --follow-redirects".dimmed()
                                    ));
                                } else if err.is_connect() {
                                    progress.println(format!(
                                        "{} {} {}",
                                        ERROR.to_string().red(),
                                        "Connection error".bold(),
                                        url
                                    ));
                                } else if err.is_request() {
                                    progress.println(format!(
                                        "{} {} {} {}",
                                        ERROR.to_string().red(),
                                        "Request error".bold(),
                                        url,
                                        format!("({})", err).dimmed()
                                    ));
                                } else {
                                    progress.println(format!(
                                        "{} {} {} {}",
                                        ERROR.to_string().red(),
                                        "Unknown Error".bold(),
                                        url,
                                        format!("({})", err).dimmed()
                                    ));
                                }
                            }
                        }
                        // Increase the index of the current chunk in the hashmap
                        indexes
                            .lock()
                            .get_mut(&previous_node.lock().data.url)
                            .unwrap()[i] += 1;
                        progress.inc(1);
                    }
                });
                handles.push(handle);
            }
        }

        // Wait for all handles to finish
        for handle in handles {
            handle.await?;
        }

        // Go to the next depth (/a/b/c -> /a/b/c/d)
        *depth.lock() += 1;
    }
    Ok(())
}
