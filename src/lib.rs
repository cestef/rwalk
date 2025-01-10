#![allow(dead_code)]

use cli::Opts;
use crossbeam::deque::{Injector, Worker};
use eyre::Result;
use std::{sync::Arc, time};

use wordlist::Wordlist;
use worker::{
    filters::{status::StatusFilter, Filter, Filtrerer},
    utils::find_task,
};

pub mod cli;
pub mod types;
pub mod wordlist;
pub mod worker;

type Task = String;

pub async fn run(opts: Opts) -> Result<()> {
    let mut wordlists = vec![];

    for path in &opts.wordlists {
        let mut wordlist = Wordlist::from_path(&path).await?;
        wordlist.dedup();
        wordlists.push(wordlist);
    }

    let global = Arc::new(Injector::<Task>::new());
    let workers = (0..num_cpus::get())
        .map(|_| Worker::<Task>::new_fifo())
        .collect::<Vec<_>>();
    let stealers = workers.iter().map(|e| e.stealer()).collect::<Vec<_>>();

    // Inject all tasks into the global queue.

    for wordlist in &wordlists {
        wordlist.inject(&global);
    }

    println!("Injected {} tasks", global.len());

    let mut handles = vec![];

    let client = reqwest::Client::new();
    let filterer = Filtrerer::new(vec![StatusFilter::construct("200-299")?]);
    for worker in workers {
        let global = global.clone();
        let stealers = stealers.clone();
        let client = client.clone();
        let opts = opts.clone();
        let filterer = filterer.clone();
        handles.push(tokio::spawn(async move {
            while let Some(task) = find_task(&worker, &global, &stealers) {
                let start = time::Instant::now();
                let res = client.get(opts.url.join(&task)?).send().await?;
                let elapsed_get = start.elapsed();
                let response = worker::utils::SendableResponse::from_response(res).await?;
                let elapsed_build = start.elapsed() - elapsed_get;
                if filterer.all(&response) {
                    println!("{}: {:?} - {:?}", task, elapsed_get, elapsed_build);
                }
            }
            eyre::Ok(())
        }));
    }
    for handle in handles {
        handle.await??;
    }

    Ok(())
}
