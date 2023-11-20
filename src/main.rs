#![allow(dead_code)]

use crate::{
    cli::OPTS,
    constants::{ERROR, INFO, SAVE_FILE, STATUS_CODES, SUCCESS, WARNING},
    tree::{Tree, TreeData},
    utils::parse_wordlists,
};
use anyhow::Result;
use colored::Colorize;
use indicatif::{HumanDuration, MultiProgress};
use parking_lot::Mutex;
use ptree::print_tree;
use reqwest::{header::HeaderMap, redirect::Policy};
use serde::{Deserialize, Serialize};
use std::{io::Write, sync::Arc, time::Duration};
use url::Url;

mod cli;
mod constants;
mod tree;
mod utils;

#[derive(Serialize, Deserialize)]
struct Save {
    tree: Arc<Mutex<Tree<TreeData>>>,
    depth: Arc<Mutex<usize>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    if !OPTS.quiet {
        utils::banner();
    }
    let depth = Arc::new(Mutex::new(0));
    let saved = std::fs::read_to_string(SAVE_FILE);
    let saved = match saved {
        Ok(saved) => {
            let saved: Save = serde_json::from_str(&saved)?;
            if let Some(root) = &saved.tree.clone().lock().root {
                if root.lock().data.url != OPTS.url {
                    None
                } else {
                    print_tree(&*root.lock())?;
                    println!(
                        "{} Found saved state crawled to depth {}",
                        INFO.to_string().blue(),
                        saved.depth.lock().to_string().blue()
                    );
                    let res = inquire::Confirm::new("Do you want to resume?")
                        .with_default(true)
                        .prompt()?;
                    if !res {
                        std::fs::remove_file(SAVE_FILE)?;
                        None
                    } else {
                        *depth.lock() = *saved.depth.lock();
                        Some(saved.tree)
                    }
                }
            } else {
                None
            }
        }
        Err(_) => None,
    };
    let has_saved = saved.is_some();
    let tree = if let Some(saved) = saved {
        saved
    } else {
        let t = Arc::new(Mutex::new(Tree::new()));
        t.lock().insert(
            TreeData {
                url: OPTS.url.clone(),
                depth: 0,
                path: Url::parse(&OPTS.url)?
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

    let words = parse_wordlists(&OPTS.wordlists);
    let words = if OPTS.case_insensitive {
        // Delete duplicates and lowercase
        let mut words = words.iter().map(|x| x.to_lowercase()).collect::<Vec<_>>();
        words.sort_unstable();
        words.dedup();
        words
    } else {
        words
    };
    if words.len() == 0 {
        println!("{} No words found in wordlists", ERROR.to_string().red());
        std::process::exit(1);
    }
    let threads = OPTS
        .threads
        .unwrap_or(num_cpus::get() * 10)
        .max(1)
        .min(words.len());
    println!(
        "{} Starting crawler with {} threads and {} words",
        INFO.to_string().blue(),
        threads,
        words.len()
    );
    let chunks = Arc::new(
        words
            .chunks(words.len() / threads)
            .map(|x| x.to_vec())
            .collect::<Vec<_>>(),
    );
    let watch = stopwatch::Stopwatch::start_new();
    let ctrlc_tree = tree.clone();
    let ctrlc_depth = depth.clone();
    let current_root: Arc<Mutex<Option<MultiProgress>>> = Arc::new(Mutex::new(None));
    let ctrlc_root = current_root.clone();
    ctrlc::set_handler(move || {
        let content = serde_json::to_string(&Save {
            tree: ctrlc_tree.clone(),
            depth: ctrlc_depth.clone(),
        });
        match content {
            Ok(content) => {
                let mut file = std::fs::File::create(SAVE_FILE).unwrap();
                file.write_all(content.as_bytes()).unwrap();
                if let Some(root) = &*ctrlc_root.lock() {
                    root.println(format!(
                        "{} Saved state to {}",
                        INFO.to_string().blue(),
                        SAVE_FILE.bold()
                    ))
                    .unwrap();
                }
            }
            Err(_) => {}
        }

        std::process::exit(0);
    })?;
    while *depth.lock() < OPTS.depth {
        let previous_nodes = tree.lock().get_nodes_at_depth(depth.lock().clone());
        let root_progress = indicatif::MultiProgress::new();
        *current_root.lock() = Some(root_progress.clone());
        let mut progresses = Vec::new();
        for node in &previous_nodes {
            let pb = root_progress.add(indicatif::ProgressBar::new(words.len() as u64))
                .with_style(
                indicatif::ProgressStyle::default_bar()
                    .template("{spinner:.blue} (ETA. {eta}) [{wide_bar}] {pos}/{len} ({per_sec:>11}) | {prefix:>3} {msg:>14.bold}")?
                    .progress_chars("█▉▊▋▌▍▎▏ "),
            )
                .with_message(format!("/{}",node.lock().data.path.trim_start_matches("/")))
                .with_prefix(format!("d={}", depth.lock()));
            pb.enable_steady_tick(Duration::from_millis(100));
            progresses.push(pb);
        }
        let mut handles = Vec::new();
        for previous_node in &previous_nodes {
            let progress = progresses.pop().unwrap();
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
            for chunk in &*chunks {
                let headers = headers.clone();
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
                let mut tree = tree.lock().clone();
                let previous_node = previous_node.clone();
                let chunk = chunk.clone();
                let client = client.clone();
                let progress = progress.clone();
                let handle = tokio::spawn(async move {
                    for word in chunk {
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
                                        if child.lock().data.path == word {
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

    println!(
        "{} Done in {}",
        INFO.to_string().blue(),
        HumanDuration(watch.elapsed())
    );

    let root = tree.lock().root.clone().unwrap().clone();

    print_tree(&*root.lock())?;
    if has_saved {
        std::fs::remove_file(SAVE_FILE)?;
    }
    if OPTS.output.is_some() {
        let output = OPTS.output.clone().unwrap();
        let file_type = output.split(".").last().unwrap_or("json");
        let mut file = std::fs::File::create(OPTS.output.clone().unwrap())?;

        match file_type {
            "json" => {
                file.write_all(serde_json::to_string(&*root.lock())?.as_bytes())?;
                file.flush()?;
            }
            "csv" => {
                let mut writer = csv::Writer::from_writer(file);
                let mut nodes = Vec::new();
                for depth in 0..*depth.lock() {
                    nodes.append(&mut tree.lock().get_nodes_at_depth(depth));
                }
                for node in nodes {
                    writer.serialize(node.lock().data.clone())?;
                }
                writer.flush()?;
            }
            "md" => {
                let mut nodes = Vec::new();
                for depth in 0..*depth.lock() {
                    nodes.append(&mut tree.lock().get_nodes_at_depth(depth));
                }
                for node in nodes {
                    let data = node.lock().data.clone();
                    let emoji = utils::get_emoji_for_status_code(data.status_code);
                    let url = data.url;
                    let path = data.path;
                    let depth = data.depth;
                    let status_code = data.status_code;
                    let line = format!(
                        "{}- [{} /{} {}]({})",
                        "  ".repeat(depth),
                        emoji,
                        path.trim_start_matches("/"),
                        if status_code == 0 {
                            "".to_string()
                        } else {
                            format!("({})", status_code)
                        },
                        url,
                    );
                    file.write_all(line.as_bytes())?;
                    file.write_all(b"\n")?;
                }
                file.flush()?;
            }
            "txt" => {
                let mut nodes = Vec::new();
                for depth in 0..*depth.lock() {
                    nodes.append(&mut tree.lock().get_nodes_at_depth(depth));
                }
                for node in nodes {
                    let data = node.lock().data.clone();
                    file.write_all(data.url.as_bytes())?;
                    file.write_all(b"\n")?;
                }
                file.flush()?;
            }
            _ => {
                println!(
                    "{} Invalid output file type",
                    ERROR.to_string().red().bold()
                );
                std::process::exit(1);
            }
        }

        println!("{} Saved to {}", SUCCESS.to_string().green(), output.bold());
    }
    // utils::show_cursor();
    std::process::exit(0);
}
