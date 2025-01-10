use crossbeam::deque::{Injector, Stealer, Worker};
use reqwest::Client;
use std::sync::Arc;
use tokio::task::JoinHandle;
use url::Url;

use crate::{
    error::Result,
    filters::Filtrerer,
    worker::utils::{self, RwalkResponse},
};

use super::handler::ResponseHandler;

pub struct WorkerPool {
    threads: usize,
    global_queue: Arc<Injector<String>>,
    client: Client,
    base_url: Url,
    filterer: Filtrerer<RwalkResponse>,
}

impl WorkerPool {
    pub fn new(
        threads: usize,
        global_queue: Arc<Injector<String>>,
        client: Client,
        base_url: Url,
        filterer: Filtrerer<RwalkResponse>,
    ) -> Self {
        Self {
            threads,
            global_queue,
            client,
            base_url,
            filterer,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let workers = self.create_workers();
        let stealers = workers.iter().map(|w| w.stealer()).collect::<Vec<_>>();
        let handles = self.spawn_workers(workers, stealers);

        for handle in handles {
            handle.await??;
        }

        Ok(())
    }

    fn create_workers(&self) -> Vec<Worker<String>> {
        (0..self.threads).map(|_| Worker::new_fifo()).collect()
    }

    fn spawn_workers(
        &self,
        workers: Vec<Worker<String>>,
        stealers: Vec<Stealer<String>>,
    ) -> Vec<JoinHandle<Result<()>>> {
        let needs_body = self.filterer.needs_body();

        let handler = ResponseHandler::new(self.filterer.clone(), needs_body);
        workers
            .into_iter()
            .map(|worker| {
                let global = self.global_queue.clone();
                let stealers = stealers.clone();
                let client = self.client.clone();
                let base_url = self.base_url.clone();
                let handler = handler.clone();

                tokio::spawn(async move {
                    while let Some(task) = utils::find_task(&worker, &global, &stealers) {
                        let start = std::time::Instant::now();
                        let res = client.get(base_url.join(&task)?).send().await?;
                        handler.handle_response(res, task, start).await?;
                    }
                    Ok(())
                })
            })
            .collect()
    }
}
