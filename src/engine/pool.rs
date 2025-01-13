use crate::{
    cli::Opts,
    error::Result,
    filters::Filterer,
    types::EngineMode,
    utils::{
        constants::DEFAULT_RESPONSE_FILTERS, throttle::DynamicThrottler, ticker::RequestTicker,
    },
    wordlist::Wordlist,
    worker::{
        filters::ResponseFilterRegistry,
        utils::{self, RwalkResponse},
    },
};

use crossbeam::deque::{Injector, Stealer, Worker};
use dashmap::DashMap as HashMap;
use reqwest::Client;
use std::sync::Arc;
use tokio::task::JoinHandle;

use url::Url;

use super::handler::{recursive::RecursiveHandler, template::TemplateHandler, ResponseHandler};

#[derive(Clone)]
pub struct WorkerPool {
    pub(crate) threads: usize,
    pub(crate) global_queue: Arc<Injector<String>>,
    pub(crate) client: Client,
    pub(crate) base_url: Url,
    pub(crate) filterer: Filterer<RwalkResponse>,
    pub(crate) mode: EngineMode,
    pub(crate) wordlists: Arc<Vec<Wordlist>>,
    throttler: Option<Arc<DynamicThrottler>>,
}

impl WorkerPool {
    pub fn new(
        threads: usize,
        global_queue: Arc<Injector<String>>,
        client: Client,
        base_url: Url,
        filterer: Filterer<RwalkResponse>,
        mode: EngineMode,
        wordlists: Vec<Wordlist>,
        rps: Option<(u64, u64)>,
        worker_count: usize,
    ) -> Self {
        Self {
            threads,
            global_queue,
            client,
            base_url,
            filterer,
            mode,
            wordlists: Arc::new(wordlists),
            throttler: rps.map(|rps| Arc::new(DynamicThrottler::new(rps.0, rps.1, worker_count))),
        }
    }

    fn create_filterer(opts: &Opts) -> Result<Filterer<RwalkResponse>> {
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
        Ok(Filterer::new(response_filters))
    }

    pub fn from_opts(opts: Opts, wordlists: Vec<Wordlist>) -> Result<Self> {
        let global_queue = Arc::new(Injector::new());
        let client = Client::new();
        let base_url = opts.url.clone();

        Ok(Self::new(
            opts.threads,
            global_queue,
            client,
            base_url,
            Self::create_filterer(&opts)?,
            opts.mode,
            wordlists,
            opts.throttle,
            opts.threads,
        ))
    }

    pub async fn run(self) -> Result<Arc<HashMap<String, RwalkResponse>>> {
        let results = Arc::new(HashMap::<String, RwalkResponse>::new());

        let workers = self.create_workers();
        let stealers = workers.iter().map(|w| w.stealer()).collect::<Vec<_>>();
        let ticker = RequestTicker::new(5);
        let handles = self.spawn_workers(workers, stealers, results.clone(), ticker.clone())?;

        tokio::spawn(async move {
            loop {
                println!("{:.2} reqs/s", ticker.get_rate());
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
        for handle in handles {
            handle.await??;
        }

        Ok(results)
    }

    fn create_workers(&self) -> Vec<Worker<String>> {
        (0..self.threads).map(|_| Worker::new_fifo()).collect()
    }

    fn spawn_workers(
        self,
        workers: Vec<Worker<String>>,
        stealers: Vec<Stealer<String>>,
        results: Arc<HashMap<String, RwalkResponse>>,
        ticker: Arc<RequestTicker>,
    ) -> Result<Vec<JoinHandle<Result<()>>>> {
        let handler: Box<dyn ResponseHandler> = match self.mode {
            EngineMode::Recursive => Box::new(RecursiveHandler::construct(self.filterer.clone())),
            EngineMode::Template => Box::new(TemplateHandler::construct(self.filterer.clone())),
        };
        let handler = Arc::new(handler);

        handler.init(&self)?;

        let needs_body = self.filterer.needs_body();
        Ok(workers
            .into_iter()
            .map(|worker| {
                let global = self.global_queue.clone();
                let stealers = stealers.clone();
                let client = self.client.clone();
                let handler = handler.clone();
                let filterer = self.filterer.clone();
                let results = results.clone();
                let ticker = ticker.clone();
                let self_ = self.clone();
                let throttler = self.throttler.clone();
                tokio::spawn(async move {
                    while let Some(task) = utils::find_task(&worker, &global, &stealers) {
                        if let Some(throttler) = throttler.as_ref() {
                            throttler.wait_for_request().await;
                        }

                        let start = std::time::Instant::now();
                        let res = client.get(task).send().await?;

                        if let Some(throttler) = throttler.as_ref() {
                            if res.status().is_server_error() || res.status() == 429 {
                                throttler.record_error();
                            } else {
                                throttler.record_success();
                            }
                        }

                        let rwalk_response =
                            RwalkResponse::from_response(res, needs_body, start).await?;
                        if filterer.all(&rwalk_response) {
                            println!("{}", rwalk_response.url);
                            handler.handle(rwalk_response.clone(), &self_)?;
                            results.insert(rwalk_response.url.to_string(), rwalk_response);
                        }
                        ticker.tick();
                    }
                    Ok(())
                })
            })
            .collect())
    }
}
