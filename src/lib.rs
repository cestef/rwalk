#![allow(dead_code, unused_macros)]

use engine::WorkerPool;

use cli::Opts;
use indicatif::HumanDuration;
use owo_colors::OwoColorize;
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

pub async fn run(opts: Opts) -> Result<()> {
    let start = std::time::Instant::now();

    // Check if the website is reachable
    if !opts.force {
        let url = opts.url.clone();
        let host = url.host_str().ok_or_else(|| error!("Invalid URL"))?;
        let scheme = url.scheme();
        reqwest::get(&format!("{scheme}://{host}"))
            .await
            .map_err(|e| RwalkError::UnreachableHost { source: e })?;
    }

    // Process wordlists
    let processor = WordlistProcessor::new(&opts);
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
    // info!("{}\n", pool.worker_config.filterer.filter.as_ref().unwrap());

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

    tree::display_url_tree(&opts.url, &results);

    success!(
        "Done in {} with an average of {} req/s",
        format!("{:#}", HumanDuration(start.elapsed())).bold(),
        rate.round().bold()
    );

    Ok(())
}
