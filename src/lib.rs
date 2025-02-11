#![allow(dead_code)]

use engine::WorkerPool as Engine;

use cli::Opts;

use utils::{constants, error, tree, types};

use wordlist::processor::WordlistProcessor;

pub mod cli;
pub mod engine;
pub mod filters;
pub mod utils;
pub mod wordlist;
pub mod worker;

pub(crate) use error::error;
pub use error::Result;

pub async fn run(opts: Opts) -> Result<f64> {
    // Process wordlists
    let processor = WordlistProcessor::new(&opts);
    let wordlists = processor.process_wordlists().await?;

    let engine = Engine::from_opts(opts, wordlists)?;
    let (results, rate) = engine.run().await?;

    tree::display_url_tree(&results);
    Ok(rate)
}
