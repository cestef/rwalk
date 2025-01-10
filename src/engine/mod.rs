use crossbeam::deque::Injector;
use reqwest::Client;
use std::sync::Arc;

use crate::{
    cli::Opts,
    error::Result,
    filters::Filtrerer,
    wordlist::{processor::WordlistProcessor, Wordlist},
};

pub mod handler;
pub mod pool;

use pool::WorkerPool;

pub struct Engine {
    opts: Opts,
    client: Client,
    global_queue: Arc<Injector<String>>,
    filterer: Filtrerer<crate::worker::utils::RwalkResponse>,
}

impl Engine {
    pub fn new(opts: Opts, filterer: Filtrerer<crate::worker::utils::RwalkResponse>) -> Self {
        Self {
            opts,
            client: Client::new(),
            global_queue: Arc::new(Injector::new()),
            filterer,
        }
    }

    pub async fn run(&self) -> Result<()> {
        // Process wordlists
        let processor = WordlistProcessor::new(&self.opts);
        let wordlists = processor.process_wordlists().await?;

        // Initialize worker pool
        let pool = WorkerPool::new(
            self.opts.threads,
            self.global_queue.clone(),
            self.client.clone(),
            self.opts.url.clone(),
            self.filterer.clone(),
        );

        // Inject tasks
        self.inject_tasks(&wordlists);

        // Run workers
        pool.run().await?;

        Ok(())
    }

    fn inject_tasks(&self, wordlists: &[Wordlist]) {
        for wordlist in wordlists {
            wordlist.inject(&self.global_queue);
        }
        println!("Injected {} tasks", self.global_queue.len());
    }
}
