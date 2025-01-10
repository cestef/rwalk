#![allow(dead_code)]

use std::{collections::HashMap, sync::Arc, time};

use crossbeam::deque::{Injector, Worker};
use tokio::task::JoinHandle;

use cli::Opts;
use constants::DEFAULT_RESPONSE_FILTERS;
use filters::Filtrerer;
use wordlist::Wordlist;
use worker::{filters::ResponseFilterRegistry, utils::find_task};

pub mod cli;
pub mod constants;
pub mod error;
pub mod filters;
pub mod types;
pub mod wordlist;
pub mod worker;

pub(crate) use error::error;
pub use error::Result;

type Task = String;

pub async fn run(opts: Opts) -> Result<()> {
    let mut wordlists = vec![];

    for path in &opts.wordlists {
        let mut wordlist = Wordlist::from_path(path).await?;
        wordlist.dedup();
        wordlists.push(wordlist);
    }

    let global = Arc::new(Injector::<Task>::new());

    let workers = (0..opts.threads)
        .map(|_| Worker::<Task>::new_fifo())
        .collect::<Vec<_>>();
    let stealers = workers.iter().map(|e| e.stealer()).collect::<Vec<_>>();

    // Inject all tasks into the global queue.
    for wordlist in &wordlists {
        wordlist.inject(&global);
    }

    println!("Injected {} tasks", global.len());

    let mut handles: Vec<JoinHandle<Result<()>>> = vec![];

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

    let client = reqwest::Client::new();

    for worker in workers {
        let global = global.clone();
        let stealers = stealers.clone();
        let client = client.clone();
        let opts = opts.clone();

        let filterer = filterer.clone();
        let needs_body = filterer.needs_body();

        handles.push(tokio::spawn(async move {
            while let Some(task) = find_task(&worker, &global, &stealers) {
                let start = time::Instant::now();
                let res = client.get(opts.url.join(&task)?).send().await?;

                let response = worker::utils::RwalkResponse::from_response(res, needs_body).await?;

                if filterer.all(&response) {
                    println!("{} ({:?})", task, start.elapsed());
                }
            }
            crate::Result::Ok(())
        }));
    }
    for handle in handles {
        handle.await??;
    }

    Ok(())
}
