use colored::Colorize;
use reqwest::{header::HeaderMap, redirect::Policy};
use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use parking_lot::Mutex;

use crate::{
    cli::OPTS,
    constants::{ERROR, STATUS_CODES, SUCCESS, WARNING},
    tree::{Tree, TreeData},
};

pub async fn start(
    depth: Arc<Mutex<usize>>,
    tree: Arc<Mutex<Tree<TreeData>>>,
    current_indexes: Arc<Mutex<HashMap<String, Vec<usize>>>>,
    chunks: Arc<Vec<Vec<String>>>,
    words: Vec<String>,
) -> Result<()> {
    while *depth.lock() < OPTS.depth {
        let previous_nodes = tree.lock().get_nodes_at_depth(depth.lock().clone());
        let root_progress = indicatif::MultiProgress::new();
        let mut progresses = HashMap::new();
        let mut handles = Vec::new();
        for previous_node in &previous_nodes {
            let mut indexes = current_indexes.lock();
            let index = match indexes.entry(previous_node.lock().data.url.clone()) {
                Occupied(entry) => entry.into_mut(),
                Vacant(entry) => entry.insert(vec![0; chunks.len()]),
            };
            let pb = root_progress.add(indicatif::ProgressBar::new((words.len()) as u64))
                .with_style(
                    indicatif::ProgressStyle::default_bar()
                        .template("{spinner:.blue} (ETA. {eta}) {wide_bar} {pos}/{len} ({per_sec:>11}) | {prefix:>3} {msg:>14.bold}")?
                        .progress_chars("█▉▊▋▌▍▎▏ "),
                    )
                .with_message(format!("/{}", previous_node.lock().data.path.trim_start_matches("/")))
                .with_prefix(format!("d={}", *depth.lock() + 1))
                .with_position(
                    // Sum of the indexes of the chunks
                    index.iter().sum::<usize>() as u64,
                );
            // pb.enable_steady_tick(Duration::from_millis(100));
            progresses.insert(previous_node.lock().data.url.clone(), pb);

            let progress = progresses.get(&previous_node.lock().data.url).unwrap();

            let mut headers = HeaderMap::new();
            let cookies = OPTS.cookies.clone();
            for cookie in &cookies {
                let mut cookie = cookie.splitn(2, "=");
                let key = cookie.next().unwrap().trim();
                let value = cookie.next().unwrap().trim();
                headers.insert(
                    reqwest::header::COOKIE,
                    format!("{}={}", key, value).parse().unwrap(),
                );
            }
            for header in &OPTS.headers {
                let mut header = header.splitn(2, ":");
                let key = header.next().unwrap().trim();
                let value = header.next().unwrap().trim();
                headers.insert(key, value.parse().unwrap());
            }
            let client = reqwest::Client::builder()
                .user_agent(
                    OPTS.user_agent
                        .clone()
                        .unwrap_or(format!("rwalk/{}", env!("CARGO_PKG_VERSION"))),
                )
                .default_headers(headers)
                .redirect(if OPTS.follow_redirects > 0 {
                    Policy::limited(OPTS.follow_redirects)
                } else {
                    Policy::none()
                })
                .timeout(std::time::Duration::from_secs(OPTS.timeout))
                .build()
                .unwrap();
            for (i, chunk) in chunks.iter().enumerate() {
                let mut tree = tree.lock().clone();
                let previous_node = previous_node.clone();
                let chunk = chunk.clone();
                let client = client.clone();
                let progress = progress.clone();
                let indexes = current_indexes.clone();
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
                        if !url.ends_with("/") {
                            url.push_str("/");
                        }
                        url.push_str(&word);
                        let sender = match OPTS.method.as_str() {
                            "GET" => client.get(&url),
                            "POST" => client
                                .post(&url)
                                .body(OPTS.data.clone().unwrap_or("".to_string())),
                            "PUT" => client
                                .put(&url)
                                .body(OPTS.data.clone().unwrap_or("".to_string())),
                            "DELETE" => client.delete(&url),
                            "HEAD" => client.head(&url),
                            "OPTIONS" => client.request(reqwest::Method::OPTIONS, &url),
                            "TRACE" => client.request(reqwest::Method::TRACE, &url),
                            "CONNECT" => client.request(reqwest::Method::CONNECT, &url),
                            _ => panic!("Invalid HTTP method"),
                        };
                        let t1 = std::time::Instant::now();
                        let response = sender.send().await;
                        let sleep = if OPTS.throttle > 0 {
                            let t2 = std::time::Instant::now();
                            let elapsed = t2 - t1;
                            let sleep = Duration::from_secs_f64(1.0 / OPTS.throttle as f64);
                            if elapsed < sleep {
                                sleep - elapsed
                            } else {
                                Duration::from_secs_f64(0.0)
                            }
                        } else {
                            Duration::from_secs_f64(0.0)
                        };
                        if sleep.as_secs_f64() > 0.0 {
                            tokio::time::sleep(sleep).await;
                        }
                        match response {
                            Ok(response) => {
                                if STATUS_CODES
                                    .iter()
                                    .any(|x| x.contains(&response.status().as_u16()))
                                {
                                    progress.println(format!(
                                        "{} {} {}",
                                        if response.status().is_success() {
                                            SUCCESS.to_string().green()
                                        } else if response.status().is_redirection() {
                                            WARNING.to_string().yellow()
                                        } else {
                                            ERROR.to_string().red()
                                        },
                                        response.status().as_str().bold(),
                                        url
                                    ));
                                    // Check if this path is already in the tree
                                    let mut found = false;
                                    for child in &previous_node.lock().children {
                                        if child.lock().data.path == *word {
                                            found = true;
                                            break;
                                        }
                                    }

                                    if !found {
                                        tree.insert(
                                            TreeData {
                                                url: url.clone(),
                                                depth: data.depth + 1,
                                                path: word.clone(),
                                                status_code: response.status().as_u16(),
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
                                } else {
                                    progress.println(format!(
                                        "{} {} {}",
                                        ERROR.to_string().red(),
                                        "Error".bold(),
                                        url
                                    ));
                                }
                            }
                        }
                        // Increase the index of the chunk in the hashmap
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
        for handle in handles {
            handle.await?;
        }

        *depth.lock() += 1;
    }
    Ok(())
}
