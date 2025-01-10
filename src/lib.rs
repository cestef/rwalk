#![allow(dead_code)]

use std::collections::HashMap;

use engine::Engine;

use cli::Opts;
use constants::DEFAULT_RESPONSE_FILTERS;
use filters::Filtrerer;

use worker::filters::ResponseFilterRegistry;

pub mod cli;
pub mod constants;
pub mod engine;
pub mod error;
pub mod filters;
pub mod types;
pub mod wordlist;
pub mod worker;

pub(crate) use error::error;
pub use error::Result;

pub async fn run(opts: Opts) -> Result<()> {
    let response_filters = DEFAULT_RESPONSE_FILTERS
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .chain(
            opts.filters
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
        )
        .collect::<HashMap<_, _>>()
        .into_iter()
        .map(|(k, v)| ResponseFilterRegistry::construct(&k, &v))
        .collect::<Result<Vec<_>>>()?;

    println!("Using response filters: {:?}", response_filters);

    let filterer = Filtrerer::new(response_filters);
    let engine = Engine::new(opts, filterer);
    engine.run().await
}
