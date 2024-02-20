use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    cli::opts::Opts,
    utils::{
        constants::{ERROR, PROGRESS_CHARS, PROGRESS_TEMPLATE, SUCCESS, WARNING},
        tree::{Tree, TreeData},
    },
};
use anyhow::Result;
use colored::Colorize;
use indicatif::ProgressBar;
use itertools::Itertools;
use log::info;
use parking_lot::Mutex;
use serde_json::json;
use tokio::task::JoinHandle;
use url::Url;

pub async fn run(
    url: String,
    opts: Opts,                       // The options passed to the program
    tree: Arc<Mutex<Tree<TreeData>>>, // The tree to be populated
    words: Vec<String>, // Each chunk is a list of strings to be passed to individual threads
    threads: usize,     // The number of threads to use
) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message(format!("Generating URLs..."));
    spinner.enable_steady_tick(Duration::from_millis(100));

    let urls: Vec<String> = if opts.permutations {
        let token_count = url.matches(opts.fuzz_key.clone().unwrap().as_str()).count();
        let combinations: Vec<_> = words.iter().permutations(token_count).collect();

        combinations
            .clone()
            .iter()
            .map(|c| {
                let mut url = url.clone();
                for word in c {
                    url = url.replace(opts.fuzz_key.clone().unwrap().as_str(), word);
                }
                url
            })
            .collect()
    } else {
        words
            .clone()
            .iter()
            .map(|c| {
                let mut url = url.clone();
                url = url.replace(opts.fuzz_key.clone().unwrap().as_str(), c);
                url
            })
            .collect()
    };
    spinner.finish_and_clear();
    info!("Generated {} URLs", urls.clone().len().to_string().bold());

    let mut handles = Vec::<JoinHandle<()>>::new();
    let progress = ProgressBar::new(urls.len() as u64).with_style(
        indicatif::ProgressStyle::default_bar()
            .template(PROGRESS_TEMPLATE)?
            .progress_chars(PROGRESS_CHARS),
    );
    let chunks = urls
        .chunks(urls.clone().len() / threads)
        .collect::<Vec<_>>();

    let client = super::client::build(&opts)?;

    for chunk in &chunks {
        let chunk = chunk.to_vec();
        let client = client.clone();
        let progress = progress.clone();
        let opts = opts.clone();
        let tree = tree.clone();
        let handle = tokio::spawn(async move {
            for url in &chunk {
                let sender = super::client::get_sender(&opts, &url, &client);

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
                        let filtered = super::filters::check(
                            &opts,
                            &text,
                            status_code,
                            t1.elapsed().as_millis(),
                            None,
                        );

                        if filtered {
                            let additions = super::filters::parse_show(&opts, &text, &response);

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
                                format!("{}ms", t1.elapsed().as_millis().to_string().bold())
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

                            let parsed = Url::parse(url).unwrap();
                            let mut tree = tree.lock().clone();
                            let root_url = tree.root.clone().unwrap().lock().data.url.clone();
                            tree.insert(
                                TreeData {
                                    url: url.clone(),
                                    depth: 0,
                                    path: parsed.path().to_string().replace(
                                        Url::parse(&root_url).unwrap().path().to_string().as_str(),
                                        "",
                                    ),
                                    status_code,
                                    extra: json!(additions),
                                },
                                tree.root.clone(),
                            );
                        }
                    }
                    Err(err) => {
                        if !opts.quiet {
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
                                    "{} {} {} {}",
                                    ERROR.to_string().red(),
                                    "Connection error".bold(),
                                    url,
                                    format!("({})", err).dimmed()
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
                }

                progress.inc(1);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    progress.finish_and_clear();

    Ok(())
}
