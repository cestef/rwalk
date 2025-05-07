#![allow(dead_code, unused_macros)]

use engine::WorkerPool;

use cli::Opts;
use indicatif::HumanDuration;
use owo_colors::OwoColorize;
use rhai::{Dynamic, Scope};
use tracing::debug;
use utils::{constants, error, tree, types};

use wordlist::processor::WordlistProcessor;

pub mod cli;
pub mod engine;
pub mod filters;
pub mod utils;
pub mod wordlist;
pub mod worker;

pub(crate) use error::error;
pub use error::*;

pub async fn run(opts: Opts, scope: Option<&mut Scope<'_>>) -> Result<()> {
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
    let wordlists = processor.process_wordlists().await?;

    let (pool, shutdown_tx) = WorkerPool::from_opts(&opts, wordlists)?;

    if opts.resume {
        pool.load_state("rwalk.state")?;
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
