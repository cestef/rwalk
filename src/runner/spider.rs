use super::{
    extract::{Document, LinkType},
    filters::is_directory,
    Runner,
};
use crate::runner::scripting::run_scripts;
use crate::{
    cli::opts::Opts,
    utils::{
        constants::{DEFAULT_DEPTH, ERROR, PROGRESS_CHARS, PROGRESS_TEMPLATE, SUCCESS, WARNING},
        tree::{Tree, TreeData, UrlType},
    },
};
use anyhow::anyhow;
use anyhow::{Context, Ok, Result};
use colored::Colorize;
use indicatif::ProgressBar;
use itertools::Itertools;
use parking_lot::Mutex;
use serde_json::json;
use std::sync::Arc;
use url::Url;

pub struct Spider {
    url: String,
    opts: Opts,
    tree: Arc<Mutex<Tree<TreeData>>>,
    threads: usize,
}

impl Spider {
    pub fn new(url: String, opts: Opts, tree: Arc<Mutex<Tree<TreeData>>>, threads: usize) -> Self {
        Self {
            url,
            opts,
            tree,
            threads,
        }
    }
}

impl Runner for Spider {
    async fn run(self) -> Result<()> {
        let base = Url::parse(&self.url)?;

        let mut current_depth = 0;
        let mut current_nodes = vec![base.clone()];
        let mut visited: Vec<TreeData> = vec![];
        let max_depth = self.opts.depth.unwrap_or(DEFAULT_DEPTH + 1);
        let pb = ProgressBar::new(0).with_style(
            indicatif::ProgressStyle::default_bar()
                .template(PROGRESS_TEMPLATE)?
                .progress_chars(PROGRESS_CHARS),
        );

        while current_depth < max_depth {
            let mut next_nodes = vec![];
            if current_nodes.is_empty() {
                break;
            }
            pb.set_length(current_nodes.len() as u64);
            pb.set_position(0);

            let client = super::client::build(&self.opts)?;
            let (tx, mut rx) = tokio::sync::mpsc::channel(current_nodes.len());
            let chunk_size = if current_nodes.len() < self.threads {
                1
            } else {
                current_nodes.len() / self.threads
            };

            let chunks = current_nodes
                .iter()
                .chunks(chunk_size)
                .into_iter()
                .map(|chunk| chunk.cloned().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            for chunk in chunks {
                let client = client.clone();
                let chunk_task = chunk.clone();
                let tx = tx.clone();
                let opts = self.opts.clone();
                tokio::spawn(async move {
                    let chunk = chunk_task;

                    for url in chunk {
                        let req = super::client::build_request(&opts, url.as_str(), &client)?;
                        let t1 = std::time::Instant::now();
                        let res = client
                            .execute(req)
                            .await
                            .context(format!("Could not fetch {}", url))?;
                        // log!(pb, "Visited <b>{}</>", url);
                        tx.send((url.clone(), res, t1.elapsed()))
                            .await
                            .context(format!("Could not send body of {} to the receiver", url))?;
                        // pb.println(format!("Visited {}", url));
                    }
                    Ok(())
                });
            }
            // pb.println(format!(
            //     "Waiting for {} nodes to be visited",
            //     current_nodes.len()
            // ));

            drop(tx);

            while let Some((url, mut response, elapsed)) = rx.recv().await {
                pb.inc(1);
                let status = response.status().as_u16();
                let mut text = String::new();

                // Read the response body into `text`
                while let std::result::Result::Ok(chunk) = response.chunk().await {
                    if let Some(chunk) = chunk {
                        text.push_str(&String::from_utf8_lossy(&chunk));
                    } else {
                        break;
                    }
                }
                let is_dir = is_directory(&response, &text);

                let filtered = super::filters::check(
                    &self.opts,
                    &pb,
                    &text,
                    elapsed.as_millis(),
                    Some(current_depth),
                    &response,
                );

                if filtered {
                    let additions = super::filters::parse_show(&self.opts, &text, &response);

                    pb.println(format!(
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
                        format!("{}ms", elapsed.as_millis().to_string().bold()).dimmed(),
                        additions.iter().fold("".to_string(), |acc, addition| {
                            format!(
                                "{} | {}: {}",
                                acc,
                                addition.key.dimmed().bold(),
                                addition.value.dimmed()
                            )
                        })
                    ));
                    let maybe_content_type = response.headers().get("content-type").map(|x| {
                        x.to_str()
                            .unwrap_or_default()
                            .split(';')
                            .next()
                            .unwrap_or_default()
                            .to_string()
                    });
                    let data = TreeData {
                        depth: current_depth,
                        path: url.path().to_string(),
                        url: url.to_string(),
                        url_type: if is_dir {
                            UrlType::Directory
                        } else if let Some(content_type) = maybe_content_type {
                            UrlType::File(content_type)
                        } else {
                            UrlType::Unknown
                        },
                        status_code: status,
                        extra: json!(additions),
                    };
                    run_scripts(&self.opts, &data, pb.clone())
                        .await
                        .map_err(|err| anyhow!("Failed to run scripts on URL {}: {}", url, err))?;
                    visited.push(data);
                    let document = Document::parse(&url, &text);

                    let links = document
                        .links(self.opts.subdomains)
                        .context(format!("Could not parse links from {}", url))?;

                    for link in links {
                        if link.link_type != LinkType::Internal {
                            continue;
                        }

                        if !visited.iter().any(|x| x.url == link.url.as_str()) {
                            next_nodes.push(link.url.clone());
                        }
                    }
                }
            }
            current_nodes = next_nodes;
            current_depth += 1;
        }

        pb.finish_and_clear();

        let mut tree = self.tree.lock();
        let root = tree.root.clone().unwrap();

        if self.opts.subdomains {
            // We need to group the visited nodes by domain
            let mut grouped: std::collections::HashMap<String, Vec<TreeData>> =
                std::collections::HashMap::new();
            for node in &visited {
                let url = Url::parse(&node.url)?;
                let domain = url.domain().unwrap().to_string();
                if let std::collections::hash_map::Entry::Vacant(e) = grouped.entry(domain.clone())
                {
                    e.insert(vec![node.clone()]);
                } else {
                    grouped.get_mut(&domain).unwrap().push(node.clone());
                }
            }

            // Insert the visited nodes into the tree by splitting their paths
            for (domain, nodes) in grouped {
                let root = tree.insert(
                    TreeData {
                        path: domain.clone(),
                        url: domain.clone(),
                        ..TreeData::default()
                    },
                    Some(root.clone()),
                );
                for node in nodes {
                    let url = Url::parse(&node.url)?;
                    let path = url.path_segments().unwrap().collect::<Vec<_>>();
                    let mut current = root.clone();
                    for segment in path {
                        let mut found = None;
                        for child in current.lock().children.clone() {
                            if child.lock().data.path == segment {
                                found = Some(child.clone());
                                break;
                            }
                        }
                        if found.is_none() {
                            let data = TreeData {
                                path: segment.to_string(),
                                ..node.clone()
                            };
                            current = tree.insert(data, Some(current.clone()));
                        } else {
                            current = found.unwrap();
                        }
                    }
                }
            }
        } else {
            // Insert the visited nodes into the tree by splitting their paths
            for node in visited {
                let url = Url::parse(&node.url)?;
                let path = url.path_segments().unwrap().collect::<Vec<_>>();
                let mut current = root.clone();
                for segment in path {
                    let mut found = None;
                    for child in current.lock().children.clone() {
                        if child.lock().data.path == segment {
                            found = Some(child.clone());
                            break;
                        }
                    }
                    if found.is_none() {
                        let data = TreeData {
                            path: segment.to_string(),
                            ..node.clone()
                        };
                        current = tree.insert(data, Some(current.clone()));
                    } else {
                        current = found.unwrap();
                    }
                }
            }
        }
        Ok(())
    }
}
