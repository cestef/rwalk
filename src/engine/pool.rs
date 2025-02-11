use crate::{
    cli::Opts,
    error::Result,
    filters::Filterer,
    types::EngineMode,
    utils::{
        constants::{DEFAULT_RESPONSE_FILTERS, PROGRESS_CHARS, PROGRESS_TEMPLATE},
        throttle::DynamicThrottler,
        ticker::RequestTickerNoReset,
    },
    wordlist::Wordlist,
    worker::{
        filters::ResponseFilterRegistry,
        utils::{self, RwalkResponse},
    },
};

use crossbeam::deque::{Injector, Stealer, Worker};
use dashmap::DashMap as HashMap;
use indicatif::ProgressBar;
use reqwest::Client;
use std::sync::Arc;
use tokio::task::JoinHandle;
use url::Url;

use super::{
    handler::{recursive::RecursiveHandler, template::TemplateHandler, ResponseHandler},
    Task,
};

// Configuration struct
#[derive(Clone)]
pub struct PoolConfig {
    pub threads: usize,
    pub base_url: Url,
    pub mode: EngineMode,
    pub rps: Option<(u64, u64)>,
    pub window: u64,
    pub error_threshold: f64,
    pub retries: usize,
}

// Worker configuration
#[derive(Clone)]
pub struct WorkerConfig {
    client: Client,
    filterer: Filterer<RwalkResponse>,
    handler: Arc<Box<dyn ResponseHandler>>,
    throttler: Option<Arc<DynamicThrottler>>,
    needs_body: bool,
}

#[derive(Clone)]
pub struct WorkerPool {
    pub config: PoolConfig,
    pub worker_config: WorkerConfig,
    pub global_queue: Arc<Injector<Task>>,
    pub wordlists: Arc<Vec<Wordlist>>,
    pub pb: ProgressBar,
    pub ticker: Arc<RequestTickerNoReset>,
}

impl WorkerPool {
    pub fn new(
        config: PoolConfig,
        global_queue: Arc<Injector<Task>>,
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
            pb: ProgressBar::new(0).with_style(
                indicatif::ProgressStyle::default_bar()
                    .template(PROGRESS_TEMPLATE)
                    .unwrap()
                    .progress_chars(PROGRESS_CHARS),
            ),
            ticker: RequestTickerNoReset::new(),
        })
    }

    pub fn from_opts(opts: Opts, wordlists: Vec<Wordlist>) -> Result<Self> {
        let config = PoolConfig {
            threads: opts.threads,
            base_url: opts.url.clone(),
            mode: opts.mode,
            rps: opts.throttle,
            window: opts.window,
            error_threshold: opts.error_threshold,
            retries: opts.retries,
        };

        let global_queue = Arc::new(Injector::new());
        let client = Self::create_client(&opts)?;
        let filterer = Self::create_filterer(&opts)?;

        Self::new(config, global_queue, client, filterer, wordlists)
    }

    fn create_client(opts: &Opts) -> Result<Client> {
        let mut builder = Client::builder();
        if opts.http1 {
            builder = builder.http1_only();
        }
        if opts.http2 {
            builder = builder.http2_prior_knowledge();
        }
        Ok(builder.build()?)
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

    pub async fn run(self) -> Result<(HashMap<String, RwalkResponse>, f64)> {
        let results = HashMap::<String, RwalkResponse>::new();
        let workers = self.create_workers();
        let stealers = Arc::new(workers.iter().map(|w| w.stealer()).collect::<Vec<_>>());

        self.worker_config.handler.init(&self)?;

        self.pb.set_length(self.global_queue.len() as u64);

        let global = self.global_queue.clone();
        let pb = self.pb.clone();
        let ticker = self.ticker.clone();
        let pb_ = pb.clone();

        let handles = self.spawn_workers(workers, stealers.clone(), results.clone())?;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                pb_.set_position(
                    pb_.length()
                        .unwrap_or_default()
                        .saturating_sub(global.len() as u64),
                );
            }
        });

        for handle in handles {
            handle.await??;
        }

        pb.finish_and_clear();

        Ok((results, ticker.get_rate()))
    }

    fn create_workers(&self) -> Vec<Worker<Task>> {
        (0..self.config.threads)
            .map(|_| Worker::new_fifo())
            .collect()
    }

    fn spawn_workers(
        self,
        workers: Vec<Worker<Task>>,
        stealers: Arc<Vec<Stealer<Task>>>,
        results: HashMap<String, RwalkResponse>,
    ) -> Result<Vec<JoinHandle<Result<()>>>> {
        workers
            .into_iter()
            .map(|worker| {
                let stealers = stealers.clone();
                let results = results.clone();
                let self_ = self.clone();

                let handle =
                    tokio::spawn(async move { self_.worker(worker, &stealers, results).await });

                Ok(handle)
            })
            .collect::<Result<Vec<_>>>()
    }

    async fn worker(
        self,
        worker: Worker<Task>,
        stealers: &Vec<Stealer<Task>>,
        results: HashMap<String, RwalkResponse>,
    ) -> Result<()> {
        while let Some(task) = utils::find_task(&worker, &self.global_queue, &stealers) {
            if let Some(throttler) = self.worker_config.throttler.as_ref() {
                throttler.wait_for_request().await;
            }

            let response = self.process_request(&task).await?;
            self.ticker.tick();
            if self.worker_config.filterer.all(&response) {
                self.worker_config.handler.handle(response.clone(), &self)?;
                results.insert(response.url.to_string(), response);
            }
        }
        Ok(())
    }

    async fn process_request(&self, task: &Task) -> Result<RwalkResponse> {
        let start = std::time::Instant::now();
        let res = self.worker_config.client.get(task.url.clone()).send().await;
        match res {
            Ok(res) => {
                let should_be_throttled = res.status().is_server_error() || res.status() == 429;
                if let Some(ref throttler) = self.worker_config.throttler {
                    if should_be_throttled {
                        throttler.record_error();
                    } else {
                        throttler.record_success();
                    }
                }
                let res = RwalkResponse::from_response(
                    res,
                    self.worker_config.needs_body,
                    start,
                    task.depth,
                )
                .await;
                res
            }
            Err(e) => {
                self.worker_config
                    .throttler
                    .as_ref()
                    .map(|t| t.record_error());
                if task.retry < self.config.retries {
                    let mut task = task.clone();
                    task.retry();
                    self.global_queue.push(task);
                    self.pb.set_length(self.global_queue.len() as u64);
                } else {
                    println!(
                        "Failed to fetch {} after {} retries",
                        task.url, self.config.retries
                    );
                }

                Err(e.into())
            }
        }
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
