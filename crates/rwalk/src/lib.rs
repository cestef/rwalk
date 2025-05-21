#![allow(dead_code, unused_macros)]

use cowstr::CowStr;
use engine::WorkerPool;

use cli::Opts;
use indicatif::HumanDuration;
use itertools::Itertools;
use owo_colors::OwoColorize;
use rhai::{Dynamic, Scope};
use tracing::debug;
use utils::{
    constants::{self, DEFAULT_WORDLIST_KEY, STATE_FILE},
    error,
    template::find_keys,
    tree,
    types::{self, EngineMode},
};

use wordlist::processor::WordlistProcessor;

pub mod cli;
pub mod engine;
pub mod filters;
pub mod utils;
pub mod wordlist;
pub mod worker;

pub(crate) use error::error;
pub use error::*;

pub async fn run(mut opts: Opts, scope: Option<&mut Scope<'_>>) -> Result<()> {
    let start = std::time::Instant::now();

    // Check if the website is reachable
    let url = opts
        .url
        .clone()
        .ok_or_else(|| error!("The URL is missing"))?;
    if !opts.force {
        let url_without_path = url
            .join("/")
            .map_err(|_| error!("Invalid URL"))?
            .to_string();
        reqwest::get(&url_without_path)
            .await
            .map_err(|e| RwalkError::UnreachableHost { source: e })?;
    }

    // Process wordlists
    let processor = WordlistProcessor::new(&opts);
    debug!("Processing wordlists: {:#?}", opts.wordlists);
    let mut wordlists = processor.process_wordlists().await?;

    let mut url_string = url.to_string();

    // Find template keys in URL and data
    let mut url_keys = find_keys(&url_string, &wordlists);
    let mut data_keys = vec![];
    if let Some(data) = &opts.data {
        data_keys = find_keys(data, &wordlists);
    }
    let header_keys = opts
        .headers
        .iter()
        .flat_map(|(_selectors, key, value)| {
            find_keys(key, &wordlists)
                .into_iter()
                .chain(find_keys(value, &wordlists))
        })
        .collect::<Vec<_>>();

    match opts.mode {
        EngineMode::Recursive => {
            let print_warning = |keys: &[(usize, CowStr)], source: &str| {
                if !keys.is_empty() {
                    warning!(
                        "{} contains the replace keyword{}: {}, this is supported with {}",
                        source,
                        if keys.len() > 1 { "s" } else { "" },
                        keys.iter().map(|e| e.1.clone()).join(", ").bold().blue(),
                        format!("{} {}", "--mode".dimmed(), "template".bold())
                    );
                }
            };

            print_warning(&url_keys, "URL");
            print_warning(&data_keys, "Data");
            print_warning(&header_keys, "Headers");

            if wordlists.len() > 1 {
                warning!(
                    "Multiple wordlists will be merged into a single one when using {}",
                    format!("{} {}", "--mode".dimmed(), "recursive".bold()),
                );

                let mut merged_wordlist = wordlists
                    .iter()
                    .map(|wordlist| wordlist.clone())
                    .reduce(|mut acc, wordlist| {
                        acc.extend(wordlist);
                        acc
                    })
                    .unwrap_or_default();

                merged_wordlist.dedup();

                wordlists = vec![merged_wordlist];
            }
        }
        EngineMode::Template => {
            if url_keys.is_empty() && data_keys.is_empty() {
                url_string =
                    url_string.trim_end_matches('/').to_string() + "/" + DEFAULT_WORDLIST_KEY;
                url_keys.push((url_string.len() - 1, DEFAULT_WORDLIST_KEY.into()));

                warning!(
                    "No template key was used {}, URL will be treated as: {}",
                    format!(
                        "(available: {})",
                        wordlists.iter().map(|e| e.key.clone()).join(", ")
                    )
                    .dimmed(),
                    url_string.bold()
                );
            }

            let used_keys: std::collections::HashSet<&str> = url_keys
                .iter()
                .chain(data_keys.iter())
                .chain(header_keys.iter())
                .map(|(_, key)| key.as_str())
                .collect();

            // Remove unused wordlists keys
            wordlists.retain(|wordlist| {
                let keep = used_keys.contains(wordlist.key.as_str());
                if !keep {
                    warning!(
                        "Wordlist {} is not used in the URL or data, removing it",
                        wordlist.key.bold().yellow()
                    );
                }
                keep
            });
        }
    }

    // Check if the URL is valid
    let url = url_string
        .parse::<url::Url>()
        .map_err(|_| error!("Invalid URL"))?;
    debug!("Parsed URL: {}", url);
    opts.url = Some(url.clone());

    let (pool, shutdown_tx) = WorkerPool::from_opts(&opts, wordlists)?;

    if opts.resume {
        pool.load_state(STATE_FILE)?;
    } else {
        pool.worker_config.handler.init(&pool)?;
    }

    info!(
        "Press {} to {} the scan",
        "Ctrl+C".bold(),
        if opts.no_save {
            "exit"
        } else {
            "save and exit"
        }
    );

    let shutdown_tx_clone = shutdown_tx.clone();

    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        println!(
            "\nReceived Ctrl+C, {} exiting...",
            if opts.no_save {
                "gracefully"
            } else {
                "saving state and"
            }
        );
        let _ = shutdown_tx_clone.send(!opts.no_save);
    });

    let rx = shutdown_tx.subscribe();

    let (results, rate) = pool.run(rx).await?;
    match opts.output.as_deref() {
        Some(e) => {
            let out = match e.extension().and_then(|e| e.to_str()) {
                Some("json") => serde_json::to_string_pretty(&*results)?,
                Some("csv") => utils::output::csv(&results),
                Some("md") => utils::output::md(&results),
                Some("html") => utils::output::html(&results, &url)?,
                _ => utils::output::txt(&results),
            };

            std::fs::write(e, out)?;

            info!("Results saved to {}", e.display().bold());
        }
        _ => {
            tree::display_url_tree(&url, &results);
        }
    }

    if let Some(scope) = scope {
        let results: rhai::Map = results
            .iter()
            .map(|e| (e.key().clone().into(), Dynamic::from(e.value().clone())))
            .collect();
        scope.set_or_push(constants::RESULTS_VAR_RHAI, results);
    }

    success!(
        "Done in {} with an average of {} req/s",
        format!("{:#}", HumanDuration(start.elapsed())).bold(),
        rate.round().bold()
    );

    Ok(())
}
