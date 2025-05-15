use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    cli::opts::Opts,
    utils::{
        constants::{ERROR, PROGRESS_CHARS, PROGRESS_TEMPLATE, SUCCESS, WARNING},
        scripting::{run_scripts, ScriptingResponse},
        tree::{Tree, TreeData, UrlType},
    },
};
use color_eyre::eyre::{eyre, Result};
use colored::Colorize;
use indicatif::ProgressBar;
use itertools::Itertools;
use log::{debug, info};
use parking_lot::Mutex;
use reqwest::Client;
use serde_json::json;
use url::Url;

use super::{filters::utils::is_directory, wordlists::ParsedWordlist, Runner};

pub struct Classic {
    url: String,
    opts: Opts,
    tree: Arc<Mutex<Tree<TreeData>>>,
    current_indexes: Arc<Mutex<HashMap<String, Vec<usize>>>>,
    words: HashMap<String, ParsedWordlist>,
    threads: usize,
}

impl Classic {
    pub fn new(
        url: String,
        opts: Opts,
        tree: Arc<Mutex<Tree<TreeData>>>,
        current_indexes: Arc<Mutex<HashMap<String, Vec<usize>>>>,
        words: HashMap<String, ParsedWordlist>,
        threads: usize,
    ) -> Self {
        Self {
            url,
            opts,
            tree,
            current_indexes,
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
        base_url: String,
        chunk: Vec<String>,
        client: Client,
        progress: ProgressBar,
        indexes: Arc<Mutex<HashMap<String, Vec<usize>>>>,
        tree: Arc<Mutex<Tree<TreeData>>>,
        opts: Opts,
        engine: Arc<rhai::Engine>,
        i: usize,
    ) -> Result<()> {
        // Extract the index safely
        let index = {
            let mut indexes = indexes.lock();
            *indexes
                .get_mut(&base_url)
                .ok_or(eyre!("Couldn't find indexes for the root node"))?
                .get(i)
                .ok_or(eyre!("Invalid index"))?
        };

        // Now it's safe to await
        for url in &chunk[index..] {
            let mut url = url.clone();
            let t1 = Instant::now();

            if !opts.distributed.is_empty() {
                let current = index % (opts.distributed.len() + 1);
                if current != 0 {
                    let host_for_this_request = &opts.distributed[current - 1];
                    let parsed_url = url::Url::parse(&url)?;
                    url = format!(
                        "{}://{}{}",
                        parsed_url.scheme(),
                        host_for_this_request,
                        parsed_url.path()
                    );
                }
            }

            let request = super::client::build_request(&opts, &url, &client)?;
            let response = client.execute(request).await;
            if let Some(ref wait) = opts.wait {
                let (min, max) = wait.split_once('-').unwrap_or_default();
                let min = min.parse::<f64>().unwrap_or(0.0);
                let max = max.parse::<f64>().unwrap_or(0.0);
                if max > 0.0 {
                    let random_wait = rand::random::<f64>() * (max - min) + min;
                    let sleep_duration = Duration::from_secs_f64(random_wait);
                    tokio::time::sleep(sleep_duration).await;
                } else {
                    progress.println(format!(
                        "{} {} {}",
                        WARNING.to_string().yellow(),
                        "Invalid wait option".bold(),
                        "Ignoring wait option".dimmed()
                    ));
                }
            }
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
                        &progress,
                        &text,
                        t1.elapsed().as_millis(),
                        None,
                        &response,
                        &engine,
                    );

                    if filtered {
                        // Parse what additional information should be shown
                        let additions =
                            super::filters::parse_show(&opts, &text, &response, &progress, &engine);

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

                        let parsed = Url::parse(&url)?;
                        let mut tree = tree.lock().clone();
                        let root_url = tree
                            .root
                            .clone()
                            .ok_or(eyre!("Failed to get root URL from tree"))?
                            .lock()
                            .data
                            .url
                            .clone();
                        let maybe_content_type = response.headers().get("content-type").map(|x| {
                            x.to_str()
                                .unwrap_or_default()
                                .split(';')
                                .next()
                                .unwrap_or_default()
                                .to_string()
                        });
                        let is_dir = is_directory(&opts, &response, text.clone(), &progress);
                        let scripting_response =
                            ScriptingResponse::from_response(response, Some(text)).await;
                        let data = TreeData {
                            url: url.clone(),
                            depth: 0,
                            path: parsed
                                .path()
                                .strip_prefix(Url::parse(&root_url)?.path())
                                .unwrap_or(parsed.path())
                                .to_string(),
                            status_code,
                            extra: json!(additions),
                            url_type: if is_dir {
                                UrlType::Directory
                            } else if let Some(content_type) = maybe_content_type {
                                UrlType::File(content_type)
                            } else {
                                UrlType::Unknown
                            },
                            response: if opts.capture {
                                Some(scripting_response.clone())
                            } else {
                                None
                            },
                        };
                        run_scripts(&opts, &data, Some(scripting_response), progress.clone())
                            .await
                            .map_err(|err| {
                                eyre!("Failed to run scripts on URL {}: {}", url, err)
                            })?;
                        tree.insert(data, tree.root.clone());
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
                        let parsed = Url::parse(&url)?;
                        let mut tree = tree.lock().clone();
                        let root_url = tree
                            .root
                            .clone()
                            .ok_or(eyre!("Failed to get root URL from tree"))?
                            .lock()
                            .data
                            .url
                            .clone();
                        let data = TreeData {
                            url: url.clone(),
                            depth: 0,
                            path: parsed
                                .path()
                                .strip_prefix(Url::parse(&root_url)?.path())
                                .unwrap_or(parsed.path())
                                .to_string(),
                            status_code: 0,
                            extra: json!([]),
                            url_type: UrlType::Unknown,
                            response: None,
                        };
                        tree.insert(data.clone(), tree.root.clone());

                        run_scripts(&opts, &data, None, progress.clone())
                            .await
                            .map_err(|err| {
                                eyre!("Failed to run scripts on URL {}: {}", url, err)
                            })?;
                    } else {
                        super::filters::utils::print_error(
                            &opts,
                            |msg| {
                                progress.println(msg);
                                Ok(())
                            },
                            &url,
                            err,
                        )?;
                    }
                }
            }
            let mut indexes = indexes.lock(); // Lock the mutex

            // Locking and working with the entry
            let entry = indexes
                .get_mut(&base_url)
                .ok_or(eyre!("Couldn't find indexes for the root node"))?
                .get_mut(i)
                .ok_or(eyre!("Invalid index"))?;

            // Increment the value at the specified index
            *entry += 1;

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

        let chunks = urls.chunks(urls.len() / self.threads).collect::<Vec<_>>();
        let mut handles = Vec::with_capacity(chunks.len());

        // Extract index safely and drop the lock early
        let index = {
            let mut indexes = self.current_indexes.lock();
            indexes
                .entry(self.url.clone())
                .or_insert_with(|| vec![0; chunks.len()])
                .clone()
        };

        let progress = ProgressBar::new(urls.len() as u64)
            .with_style(
                indicatif::ProgressStyle::default_bar()
                    .template(PROGRESS_TEMPLATE)?
                    .progress_chars(PROGRESS_CHARS),
            )
            .with_position(index.iter().sum::<usize>() as u64);

        progress.enable_steady_tick(Duration::from_millis(100));

        let client = super::client::build(&self.opts)?;
        let mut engine = rhai::Engine::new();
        engine.build_type::<ScriptingResponse>();
        let engine_opts = self.opts.clone();
        let engine_progress = progress.clone();
        engine.on_print(move |s| {
            if !engine_opts.quiet {
                engine_progress.println(s);
            }
        });
        let engine = Arc::new(engine);
        for (i, chunk) in chunks.iter().enumerate() {
            let base_url = self.url.clone();
            let chunk = chunk.to_vec();
            let client = client.clone();
            let progress = progress.clone();
            let indexes = self.current_indexes.clone();
            let tree = self.tree.clone();
            let opts = self.opts.clone();
            let engine = engine.clone();
            let res = tokio::spawn(async move {
                Self::process_chunk(
                    base_url.clone(),
                    chunk,
                    client,
                    progress,
                    indexes,
                    tree,
                    opts,
                    engine,
                    i,
                )
                .await
            });
            handles.push(res);
        }

        for handle in handles {
            let res = handle
                .await
                .map_err(|err| eyre!("Failed to receive result from worker thread: {}", err))?;
            if res.is_err() {
                return Err(res.err().unwrap());
            }
        }

        progress.finish_and_clear();

        Ok(())
    }
}
