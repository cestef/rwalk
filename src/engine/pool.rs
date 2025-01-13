use crate::{
    cli::Opts,
    error::Result,
    filters::Filterer,
    types::EngineMode,
    utils::{constants::DEFAULT_RESPONSE_FILTERS, throttle::DynamicThrottler},
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

// New type for thread count
#[derive(Clone, Copy)]
pub struct ThreadCount(pub usize);

// Configuration struct
#[derive(Clone)]
pub struct PoolConfig {
    pub threads: ThreadCount,
    pub base_url: Url,
    pub mode: EngineMode,
    pub rps: Option<(u64, u64)>,
    pub window: u64,
    pub error_threshold: f64,
}

// Worker configuration
#[derive(Clone)]
struct WorkerConfig {
    client: Client,
    filterer: Filterer<RwalkResponse>,
    handler: Arc<Box<dyn ResponseHandler>>,
    throttler: Option<Arc<DynamicThrottler>>,
    needs_body: bool,
}

#[derive(Clone)]
pub struct WorkerPool {
    pub(crate) config: PoolConfig,
    worker_config: WorkerConfig,
    pub(crate) global_queue: Arc<Injector<String>>,
    pub(crate) wordlists: Arc<Vec<Wordlist>>,
}

impl WorkerPool {
    pub fn new(
        config: PoolConfig,
        global_queue: Arc<Injector<String>>,
        client: Client,
        filterer: Filterer<RwalkResponse>,
        wordlists: Vec<Wordlist>,
    ) -> Result<Self> {
        let handler: Box<dyn ResponseHandler> =
            match config.mode {
                EngineMode::Recursive => Box::new(RecursiveHandler::construct(filterer.clone()))
                    as Box<dyn ResponseHandler>,
                EngineMode::Template => Box::new(TemplateHandler::construct(filterer.clone()))
                    as Box<dyn ResponseHandler>,
            };

        let worker_config = WorkerConfig {
            client,
            filterer: filterer.clone(),
            handler: Arc::new(handler),
            throttler: config.rps.map(|(min, max)| {
                Arc::new(DynamicThrottler::new(
                    min,
                    max,
                    config.threads.0,
                    config.window,
                    config.error_threshold,
                ))
            }),
            needs_body: filterer.needs_body(),
        };

        Ok(Self {
            config,
            worker_config,
            global_queue,
            wordlists: Arc::new(wordlists),
        })
    }

    pub fn from_opts(opts: Opts, wordlists: Vec<Wordlist>) -> Result<Self> {
        let config = PoolConfig {
            threads: ThreadCount(opts.threads),
            base_url: opts.url.clone(),
            mode: opts.mode,
            rps: opts.throttle,
            window: opts.window,
            error_threshold: opts.error_threshold,
        };

        let global_queue = Arc::new(Injector::new());
        let client = Client::new();
        let filterer = Self::create_filterer(&opts)?;

        Self::new(config, global_queue, client, filterer, wordlists)
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

    pub async fn run(self) -> Result<HashMap<String, RwalkResponse>> {
        let results = HashMap::<String, RwalkResponse>::new();
        let workers = self.create_workers();
        let stealers = workers.iter().map(|w| w.stealer()).collect::<Vec<_>>();

        let handles = self.spawn_workers(workers, stealers, results.clone())?;

        for handle in handles {
            handle.await??;
        }

        Ok(results)
    }

    fn create_workers(&self) -> Vec<Worker<String>> {
        (0..self.config.threads.0)
            .map(|_| Worker::new_fifo())
            .collect()
    }

    fn spawn_workers(
        self,
        workers: Vec<Worker<String>>,
        stealers: Vec<Stealer<String>>,
        results: HashMap<String, RwalkResponse>,
    ) -> Result<Vec<JoinHandle<Result<()>>>> {
        self.worker_config.handler.init(&self)?;

        Ok(workers
            .into_iter()
            .map(|worker| {
                let stealers = stealers.clone();
                let results = results.clone();
                let self_ = self.clone();
                tokio::spawn(async move { self_.worker(worker, stealers, results).await })
            })
            .collect())
    }
    async fn worker(
        self,
        worker: Worker<String>,
        stealers: Vec<Stealer<String>>,
        results: HashMap<String, RwalkResponse>,
    ) -> Result<()> {
        while let Some(task) = utils::find_task(&worker, &self.global_queue, &stealers) {
            if let Some(throttler) = self.worker_config.throttler.as_ref() {
                throttler.wait_for_request().await;
            }

            let response = self.process_request(&task).await?;

            if self.worker_config.filterer.all(&response) {
                println!("{}", response.url);
                self.worker_config.handler.handle(response.clone(), &self)?;
                results.insert(response.url.to_string(), response);
            }
        }
        Ok(())
    }

    async fn process_request(&self, task: &str) -> Result<RwalkResponse> {
        let start = std::time::Instant::now();
        let res = self.worker_config.client.get(task).send().await?;

        if let Some(throttler) = self.worker_config.throttler.as_ref() {
            if res.status().is_server_error() || res.status() == 429 {
                throttler.record_error();
            } else {
                throttler.record_success();
            }
        }

        RwalkResponse::from_response(res, self.worker_config.needs_body, start).await
    }
}

struct WorkerTask {
    worker: Worker<String>,
    global_queue: Arc<Injector<String>>,
    stealers: Vec<Stealer<String>>,
    config: WorkerConfig,
    results: HashMap<String, RwalkResponse>,
    pool: WorkerPool,
}
