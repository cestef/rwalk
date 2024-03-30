use std::{
    collections::HashMap,
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
use anyhow::{anyhow, Result};
use colored::Colorize;
use indicatif::ProgressBar;
use itertools::Itertools;
use log::{debug, info};
use parking_lot::Mutex;
use reqwest::Client;
use serde_json::json;
use url::Url;

use super::{wordlists::ParsedWordlist, Runner};

pub struct Classic {
    url: String,
    opts: Opts,
    tree: Arc<Mutex<Tree<TreeData>>>,
    words: HashMap<String, ParsedWordlist>,
    threads: usize,
}

impl Classic {
    pub fn new(
        url: String,
        opts: Opts,
        tree: Arc<Mutex<Tree<TreeData>>>,
        words: HashMap<String, ParsedWordlist>,
        threads: usize,
    ) -> Self {
        Self {
            url,
            opts,
            tree,
            words,
            threads,
        }
    }

    /// Generate all possible URLs using a cartesian product of the wordlists
    fn generate_urls(&self) -> Vec<String> {
        let products = self
            .words
            .iter()
            .map(|(k, ParsedWordlist { words: v, .. })| {
                v.iter().map(|w| (k, w)).collect::<Vec<_>>()
            })
            .multi_cartesian_product()
            .collect::<Vec<_>>();
        let mut urls = vec![];
        for product in &products {
            let mut url = self.url.clone();
            for (k, v) in product {
                url = url.replace(*k, v);
            }
            urls.push(url);
        }
        urls
    }

    async fn process_chunk(
        chunk: Vec<String>,
        client: Client,
        progress: ProgressBar,
        tree: Arc<Mutex<Tree<TreeData>>>,
        opts: Opts,
    ) -> Result<()> {
        for url in &chunk {
            let t1 = Instant::now();

            let request = super::client::build_request(&opts, url, &client)?;

            let response = client.execute(request).await;

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

                    // Read the response body into `text`
                    while let Ok(chunk) = response.chunk().await {
                        if let Some(chunk) = chunk {
                            text.push_str(&String::from_utf8_lossy(&chunk));
                        } else {
                            break;
                        }
                    }

                    // Check if the response is filtered (`true` means we keep it)
                    let filtered = super::filters::check(
                        &opts,
                        &text,
                        response.headers(),
                        status_code,
                        t1.elapsed().as_millis(),
                        None,
                        &response,
                    );

                    if filtered {
                        // Parse what additional information should be shown
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
                            format!("{}ms", t1.elapsed().as_millis().to_string().bold()).dimmed(),
                            additions.iter().fold("".to_string(), |acc, addition| {
                                format!(
                                    "{} | {}: {}",
                                    acc,
                                    addition.key.dimmed().bold(),
                                    addition.value.dimmed()
                                )
                            })
                        ));

                        let parsed = Url::parse(url)?;
                        let mut tree = tree.lock().clone();
                        let root_url = tree
                            .root
                            .clone()
                            .ok_or(anyhow!("Failed to get root URL from tree"))?
                            .lock()
                            .data
                            .url
                            .clone();
                        tree.insert(
                            TreeData {
                                url: url.clone(),
                                depth: 0,
                                path: parsed.path().to_string().replace(
                                    Url::parse(&root_url)?.path().to_string().as_str(),
                                    "",
                                ),
                                status_code,
                                extra: json!(additions),
                                is_dir: false,
                            },
                            tree.root.clone(),
                        );
                    }
                }
                Err(err) => {
                    // Check if the error is a connection error and the user specified to consider it as a hit
                    if opts.hit_connection_errors && err.is_connect() {
                        progress.println(format!(
                            "{} {} {} {}",
                            SUCCESS.to_string().green(),
                            "Connection error".bold(),
                            url,
                            format!("{}ms", t1.elapsed().as_millis().to_string().bold()).dimmed()
                        ));
                        let parsed = Url::parse(url)?;
                        let mut tree = tree.lock().clone();
                        let root_url = tree
                            .root
                            .clone()
                            .ok_or(anyhow!("Failed to get root URL from tree"))?
                            .lock()
                            .data
                            .url
                            .clone();

                        tree.insert(
                            TreeData {
                                url: url.clone(),
                                depth: 0,
                                path: parsed.path().to_string().replace(
                                    Url::parse(&root_url)?.path().to_string().as_str(),
                                    "",
                                ),
                                status_code: 0,
                                extra: json!([]),
                                //TODO: is_dir
                                is_dir: false,
                            },
                            tree.root.clone(),
                        );
                    } else {
                        super::filters::print_error(&opts, &progress, url, err);
                    }
                }
            }
            progress.inc(1);
        }

        Ok(())
    }
}

impl Runner for Classic {
    async fn run(self) -> Result<()> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Generating URLs...".to_string());
        spinner.enable_steady_tick(Duration::from_millis(100));

        let urls: Vec<String> = self.generate_urls();
        spinner.finish_and_clear();
        if !self.opts.quiet {
            info!("Generated {} URLs", urls.len().to_string().bold());
        }
        debug!("URLs: {:?}", urls);

        let progress = ProgressBar::new(urls.len() as u64).with_style(
            indicatif::ProgressStyle::default_bar()
                .template(PROGRESS_TEMPLATE)?
                .progress_chars(PROGRESS_CHARS),
        );

        progress.enable_steady_tick(Duration::from_millis(100));
        let chunks = urls.chunks(urls.len() / self.threads).collect::<Vec<_>>();
        let mut handles = Vec::with_capacity(chunks.len());

        let client = super::client::build(&self.opts)?;

        for chunk in &chunks {
            let chunk = chunk.to_vec();
            let client = client.clone();
            let progress = progress.clone();
            let tree = self.tree.clone();
            let opts = self.opts.clone();
            let res = tokio::spawn(async move {
                Self::process_chunk(chunk, client, progress, tree, opts).await
            });
            handles.push(res);
        }

        for handle in handles {
            let res = handle
                .await
                .map_err(|err| anyhow!("Failed to receive result from worker thread: {}", err))?;
            if res.is_err() {
                return Err(res.err().unwrap());
            }
        }

        progress.finish_and_clear();

        Ok(())
    }
}
