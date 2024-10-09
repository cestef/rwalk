use colored::Colorize;
use indicatif::MultiProgress;
use serde_json::json;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::JoinHandle;

use color_eyre::eyre::{eyre, Result};
use parking_lot::Mutex;

use crate::{
    cli::opts::Opts,
    utils::{
        constants::{DEFAULT_DEPTH, ERROR, PROGRESS_CHARS, PROGRESS_TEMPLATE, SUCCESS, WARNING},
        scripting::{run_scripts, ScriptingResponse},
        tree::{Tree, TreeData, TreeNode, UrlType},
    },
};

use super::filters::utils::is_directory;

pub struct Recursive {
    opts: Opts,
    depth: Arc<Mutex<usize>>,
    tree: Arc<Mutex<Tree<TreeData>>>,
    current_indexes: Arc<Mutex<HashMap<String, Vec<usize>>>>,
    chunks: Arc<Vec<Vec<String>>>,
}

impl super::Runner for Recursive {
    async fn run(self) -> Result<()> {
        while *self.depth.lock() < self.opts.depth.unwrap_or(DEFAULT_DEPTH) {
            let previous_nodes = self.tree.lock().get_nodes_at_depth(*self.depth.lock());

            let mut handles = Vec::new();
            let mut progresses = HashMap::new();
            let depth = self.depth.clone();
            let root_progress = MultiProgress::new();
            // Create a progress bar for each previous node
            for previous_node in &previous_nodes {
                let root_progress = root_progress.clone();
                if previous_node.lock().data.url_type != UrlType::Directory
                    && !self.opts.force_recursion
                {
                    log::debug!("Skipping not-directory {}", previous_node.lock().data.url);
                    continue;
                }
                let depth = depth.clone();
                let mut indexes = self.current_indexes.lock();
                let index = indexes
                    .entry(previous_node.lock().data.url.clone())
                    .or_insert_with(|| vec![0; self.chunks.len()]);
                let pb = root_progress
                    .add(indicatif::ProgressBar::new(
                        (self.chunks.iter().map(|chunk| chunk.len()).sum::<usize>()) as u64,
                    ))
                    .with_style(
                        indicatif::ProgressStyle::default_bar()
                            .template(PROGRESS_TEMPLATE)?
                            .progress_chars(PROGRESS_CHARS),
                    )
                    .with_message(format!(
                        "/{}",
                        previous_node.lock().data.path.trim_start_matches('/')
                    ))
                    .with_prefix(format!("d={}", *depth.lock()))
                    .with_position(index.iter().sum::<usize>() as u64);
                pb.enable_steady_tick(Duration::from_millis(100));

                progresses.insert(previous_node.lock().data.url.clone(), pb);

                let progress = progresses
                    .get(&previous_node.lock().data.url)
                    .ok_or(eyre!("Failed to get progress bar"))?
                    .clone();

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
                for (i, chunk) in self.chunks.iter().enumerate() {
                    let tree = self.tree.clone();
                    let previous_node = previous_node.clone();
                    let chunk = chunk.clone();
                    let client = client.clone();
                    let progress = progress.clone();
                    let indexes = self.current_indexes.clone();
                    let opts = self.opts.clone();
                    let depth = depth.clone();
                    let root_progress = root_progress.clone();
                    let engine = engine.clone();
                    let chunk_handle: JoinHandle<Result<()>> = tokio::spawn(async move {
                        let previous_node = previous_node.clone();
                        Self::process_chunk(
                            chunk,
                            client,
                            progress,
                            root_progress.clone(),
                            tree,
                            opts,
                            depth,
                            previous_node.clone(),
                            indexes,
                            engine,
                            i,
                        )
                        .await
                    });
                    handles.push(chunk_handle);
                }
            }

            for handle in handles {
                let res = handle
                    .await
                    .map_err(|err| eyre!("Failed to receive result from worker thread: {}", err))?;
                if res.is_err() {
                    return Err(res.err().unwrap());
                }
            }

            // Go to the next depth (/a/b/c -> /a/b/c/d)
            *depth.lock() += 1;
        }
        Ok(())
    }
}

impl Recursive {
    pub fn new(
        opts: Opts,
        depth: Arc<Mutex<usize>>,
        tree: Arc<Mutex<Tree<TreeData>>>,
        current_indexes: Arc<Mutex<HashMap<String, Vec<usize>>>>,
        chunks: Arc<Vec<Vec<String>>>,
    ) -> Self {
        Self {
            opts,
            depth,
            tree,
            current_indexes,
            chunks,
        }
    }
    #[allow(clippy::too_many_arguments)]
    async fn process_chunk(
        chunk: Vec<String>,
        client: reqwest::Client,
        progress: indicatif::ProgressBar,
        root_progress: indicatif::MultiProgress,
        tree: Arc<Mutex<Tree<TreeData>>>,
        opts: Opts,
        depth: Arc<Mutex<usize>>,
        previous_node: Arc<Mutex<TreeNode<TreeData>>>,
        indexes: Arc<Mutex<HashMap<String, Vec<usize>>>>,
        engine: Arc<rhai::Engine>,
        i: usize,
    ) -> Result<()> {
        while indexes
            .lock()
            .get_mut(&previous_node.lock().data.url)
            .ok_or(eyre!("Couldn't find indexes for the previous node"))?[i]
            < chunk.len()
        {
            let index = indexes
                .lock()
                .get_mut(&previous_node.lock().data.url)
                .ok_or(eyre!("Couldn't find indexes for the previous node"))?[i];

            let word = chunk[index].clone();
            let data = previous_node.lock().data.clone();

            let mut url = data.url.clone();
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
            match url.ends_with('/') {
                true => url.push_str(&word),
                false => url.push_str(&format!("/{}", word)),
            }

            let request = super::client::build_request(&opts, &url, &client)?;

            let t1 = Instant::now();

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
                    while let Ok(chunk) = response.chunk().await {
                        if let Some(chunk) = chunk {
                            text.push_str(&String::from_utf8_lossy(&chunk));
                        } else {
                            break;
                        }
                    }
                    let is_dir = is_directory(&opts, &response, text.clone(), &progress);

                    let filtered = super::filters::check(
                        &opts,
                        &progress,
                        &text,
                        t1.elapsed().as_millis(),
                        Some(*depth.lock()),
                        &response,
                        &engine,
                    );

                    if filtered {
                        let additions =
                            super::filters::parse_show(&opts, &text, &response, &progress, &engine);

                        root_progress.println(format!(
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
                        ))?;
                        // Check if this path is already in the tree
                        if !previous_node
                            .lock()
                            .children
                            .iter()
                            .any(|child| child.lock().data.path == *word)
                        {
                            let maybe_content_type =
                                response.headers().get("content-type").map(|x| {
                                    x.to_str()
                                        .unwrap_or_default()
                                        .split(';')
                                        .next()
                                        .unwrap_or_default()
                                        .to_string()
                                });
                            let scripting_response =
                                ScriptingResponse::from_response(response, Some(text)).await;
                            run_scripts(
                                &opts,
                                &data,
                                Some(scripting_response.clone()),
                                progress.clone(),
                            )
                            .await
                            .map_err(|err| {
                                eyre!("Failed to run scripts on URL {}: {}", url, err)
                            })?;
                            tree.lock().insert(
                                TreeData {
                                    url: url.clone(),
                                    depth: data.depth + 1,
                                    path: word.clone(),
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
                                        Some(scripting_response)
                                    } else {
                                        None
                                    },
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
                    if opts.hit_connection_errors && err.is_connect() {
                        root_progress.println(format!(
                            "{} {} {} {}",
                            SUCCESS.to_string().green(),
                            "Connection error".bold(),
                            url,
                            format!("{}ms", t1.elapsed().as_millis().to_string().bold()).dimmed()
                        ))?;
                        if !previous_node
                            .lock()
                            .children
                            .iter()
                            .any(|child| child.lock().data.path == *word)
                        {
                            run_scripts(&opts, &data, None, progress.clone())
                                .await
                                .map_err(|err| {
                                    eyre!("Failed to run scripts on URL {}: {}", url, err)
                                })?;
                            tree.lock().insert(
                                TreeData {
                                    url: url.clone(),
                                    depth: data.depth + 1,
                                    path: word.clone(),
                                    status_code: 0,
                                    extra: json!([]),
                                    url_type: UrlType::Unknown,
                                    response: None,
                                },
                                Some(previous_node.clone()),
                            );
                        } else {
                            root_progress.println(format!(
                                "{} {} {}",
                                WARNING.to_string().yellow(),
                                "Already in tree".bold(),
                                url
                            ))?;
                        }
                    } else {
                        super::filters::utils::print_error(
                            &opts,
                            |msg| {
                                root_progress.println(msg)?;
                                Ok(())
                            },
                            &url,
                            err,
                        )?;
                    }
                }
            }
            // Increase the index of the current chunk in the hashmap
            indexes
                .lock()
                .get_mut(&previous_node.lock().data.url)
                .ok_or(eyre!("Couldn't find indexes for the previous node"))?[i] += 1;
            progress.inc(1);
        }

        Ok(())
    }
}
