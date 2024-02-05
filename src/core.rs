use colored::Colorize;
use reqwest::{
    header::{HeaderMap, HeaderName},
    redirect::Policy,
    Proxy,
};
use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Result;
use parking_lot::Mutex;

use crate::{
    cli::Opts,
    constants::{ERROR, SUCCESS, WARNING},
    tree::{Tree, TreeData},
    utils::{is_response_filtered, should_filter},
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

            let pb = root_progress.add(indicatif::ProgressBar::new((words.len()) as u64))
                .with_style(
                    indicatif::ProgressStyle::default_bar()
                        .template("{spinner:.blue} (ETA. {eta}) {wide_bar} {pos}/{len} ({per_sec:>11}) | {prefix:>3} {msg:>14.bold}")?
                        .progress_chars("█▉▊▋▌▍▎▏ "),
                    )
                .with_message(format!("/{}", previous_node.lock().data.path.trim_start_matches("/")))
                .with_prefix(format!("d={}", *depth.lock() + 1))
                .with_position(
                    index.iter().sum::<usize>() as u64,
                );
            pb.enable_steady_tick(Duration::from_millis(100));
            progresses.insert(previous_node.lock().data.url.clone(), pb);

            let progress = progresses.get(&previous_node.lock().data.url).unwrap();

            let mut headers = HeaderMap::new();
            opts.headers.clone().iter().for_each(|header| {
                let mut header = header.splitn(2, ":");
                let key = header.next().unwrap().trim();
                let value = header.next().unwrap().trim();
                headers.insert(key.parse::<HeaderName>().unwrap(), value.parse().unwrap());
            });
            opts.cookies.clone().iter().for_each(|cookie| {
                let mut cookie = cookie.splitn(2, "=");
                let key = cookie.next().unwrap().trim();
                let value = cookie.next().unwrap().trim();
                headers.extend(vec![(
                    reqwest::header::COOKIE,
                    format!("{}={}", key, value).parse().unwrap(),
                )]);
            });
            let client = reqwest::Client::builder()
                .user_agent(
                    opts.user_agent
                        .clone()
                        .unwrap_or(format!("rwalk/{}", env!("CARGO_PKG_VERSION"))),
                )
                .default_headers(headers)
                .redirect(if opts.follow_redirects.unwrap() > 0 {
                    Policy::limited(opts.follow_redirects.unwrap())
                } else {
                    Policy::none()
                })
                .timeout(std::time::Duration::from_secs(opts.timeout.unwrap() as u64));

            let client = if let Some(proxy) = opts.proxy.clone() {
                let proxy = Proxy::all(proxy)?;
                if let Some(auth) = opts.proxy_auth.clone() {
                    let mut auth = auth.splitn(2, ":");
                    let username = auth.next().unwrap().trim();
                    let password = auth.next().unwrap().trim();

                    let proxy = proxy.basic_auth(username, password);
                    client.proxy(proxy)
                } else {
                    client.proxy(proxy)
                }
            } else {
                client
            };

            let client = client.build()?;
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
                        if !url.ends_with("/") {
                            url.push_str("/");
                        }
                        url.push_str(&word);
                        let sender = match opts.method.clone().unwrap().as_str() {
                            "GET" => client.get(&url),
                            "POST" => client
                                .post(&url)
                                .body(opts.data.clone().unwrap_or("".to_string())),
                            "PUT" => client
                                .put(&url)
                                .body(opts.data.clone().unwrap_or("".to_string())),
                            "DELETE" => client.delete(&url),
                            "HEAD" => client.head(&url),
                            "OPTIONS" => client.request(reqwest::Method::OPTIONS, &url),
                            "TRACE" => client.request(reqwest::Method::TRACE, &url),
                            "CONNECT" => client.request(reqwest::Method::CONNECT, &url),
                            _ => panic!("Invalid HTTP method"),
                        };
                        let t1 = std::time::Instant::now();
                        let response = sender.send().await;
                        let sleep = if opts.throttle.unwrap() > 0 {
                            let t2 = std::time::Instant::now();
                            let elapsed = t2 - t1;
                            let sleep =
                                Duration::from_secs_f64(1.0 / opts.throttle.unwrap() as f64);
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
                            Ok(mut response) => {
                                let status_code = response.status().as_u16();
                                let filtered = if should_filter(&opts) {
                                    let mut text = String::new();
                                    while let Ok(chunk) = response.chunk().await {
                                        if let Some(chunk) = chunk {
                                            text.push_str(&String::from_utf8_lossy(&chunk));
                                        } else {
                                            break;
                                        }
                                    }
                                    is_response_filtered(
                                        &opts,
                                        &text,
                                        status_code,
                                        t1.elapsed().as_millis() as u16,
                                    )
                                } else {
                                    true
                                };

                                if filtered {
                                    progress.println(format!(
                                        "{} {} {} {}",
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
                                        .dimmed()
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

        // Go to the next depth (/a/b/c -> /a/b/c/...)
        *depth.lock() += 1;
    }
    Ok(())
}
