use crate::{
    cli::Opts,
    error::Result,
    filters::Filterer,
    types::EngineMode,
    utils::{
        bell,
        constants::{
            DEFAULT_RESPONSE_FILTER, PROGRESS_CHARS, PROGRESS_TEMPLATE, PROGRESS_UPDATE_INTERVAL,
        },
        format::WARNING,
        throttle::SimpleThrottler,
        ticker::RequestTickerNoReset,
        types::IntRange,
    },
    wordlist::Wordlist,
    worker::{
        filters::ResponseFilterRegistry,
        utils::{self, RwalkResponse},
    },
    RwalkError,
};

use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use dashmap::DashMap as HashMap;
use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
    sync::Arc,
};
use tokio::sync::broadcast;
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
    pub rps: Option<u64>,
    pub retries: usize,
    pub retry_codes: Vec<IntRange<u16>>,
    pub force_recursion: bool,
    pub show: Vec<String>,
    pub max_depth: usize,
    pub bell: bool,
    pub method: reqwest::Method,
}

// Worker configuration
#[derive(Clone)]
pub struct WorkerConfig {
    pub client: Client,
    pub filterer: Filterer<RwalkResponse>,
    pub handler: Arc<Box<dyn ResponseHandler>>,
    pub throttler: Option<Arc<SimpleThrottler>>,
    pub needs_body: bool,
}

#[derive(Clone)]
pub struct WorkerPool {
    pub config: PoolConfig,
    pub worker_config: WorkerConfig,
    pub global_queue: Arc<Injector<Task>>,
    pub wordlists: Arc<Vec<Wordlist>>,
    pub pb: ProgressBar,
    pub ticker: Arc<RequestTickerNoReset>,
    pub results: Arc<HashMap<String, RwalkResponse>>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkerState {
    pending_tasks: Vec<Task>,
    completed_results: Vec<(String, RwalkResponse)>,
    base_url: Url,
}

impl WorkerPool {
    pub fn save_state<P: AsRef<Path>>(
        path: P,
        global_queue: Arc<Injector<Task>>,
        results: Arc<HashMap<String, RwalkResponse>>,
        base_url: Url,
    ) -> Result<()> {
        // Collect pending tasks from the global queue
        let mut pending_tasks = Vec::new();
        while let Steal::Success(task) = global_queue.steal() {
            pending_tasks.push(task);
        }

        // Convert DashMap to Vec for serialization
        let completed_results: Vec<(String, RwalkResponse)> = results
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        let state = WorkerState {
            pending_tasks,
            completed_results,
            base_url,
        };

        let writer = BufWriter::new(File::create(path)?);
        serde_json::to_writer(writer, &state)?;
        Ok(())
    }

    pub fn load_state<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file =
            File::open(path).map_err(|e| crate::error!(source = e, "Failed to open state file"))?;
        let reader = BufReader::new(file);
        let state: WorkerState = serde_json::from_reader(reader)?;

        if state.base_url != self.config.base_url {
            return Err(crate::error!(
                "State file does not match the current base URL"
            ));
        }

        // Restore pending tasks to global queue
        for task in state.pending_tasks {
            self.global_queue.push(task);
        }

        // Restore completed results
        for (url, response) in state.completed_results {
            self.results.insert(url, response);
        }

        // Update progress bar
        self.pb.set_length(self.global_queue.len() as u64);

        Ok(())
    }
    pub fn new(
        config: PoolConfig,
        global_queue: Arc<Injector<Task>>,
        client: Client,
        filterer: Filterer<RwalkResponse>,
        wordlists: Vec<Wordlist>,
    ) -> Result<(Self, broadcast::Sender<bool>)> {
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
            throttler: config.rps.map(|max| Arc::new(SimpleThrottler::new(max))),
            needs_body: filterer.needs_body()?, // Precompute if any filter needs body
        };

        let (shutdown_tx, _) = broadcast::channel(1);

        Ok((
            Self {
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
                results: Arc::new(HashMap::new()),
            },
            shutdown_tx,
        ))
    }

    pub fn from_opts(
        opts: &Opts,
        wordlists: Vec<Wordlist>,
    ) -> Result<(Self, broadcast::Sender<bool>)> {
        let config = PoolConfig {
            threads: opts.threads,
            base_url: opts
                .url
                .clone()
                .ok_or_else(|| crate::error!("No URL provided (This should never happen)"))?,
            mode: opts.mode,
            rps: opts.throttle,
            force_recursion: opts.force_recursion,
            retries: opts.retries,
            retry_codes: opts.retry_codes.clone(),
            show: opts.show.clone(),
            max_depth: opts.depth.overflowing_sub(1).0,
            bell: opts.bell,
            method: opts.method.into(),
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
        let filter = if opts.filters.is_empty() {
            DEFAULT_RESPONSE_FILTER.to_string()
        } else {
            opts.filters.join(" & ")
        };
        let filter = ResponseFilterRegistry::construct(&filter)?;
        Ok(Filterer::new(Some(filter)))
    }

    pub async fn run(
        self,
        mut shutdown_rx: broadcast::Receiver<bool>,
    ) -> Result<(Arc<HashMap<String, RwalkResponse>>, f64)> {
        let workers = self.create_workers();
        let stealers = Arc::new(workers.iter().map(|w| w.stealer()).collect::<Vec<_>>());

        self.pb.set_length(self.global_queue.len() as u64);

        let global = self.global_queue.clone();
        let global_ = global.clone();
        let pb = self.pb.clone();
        let ticker = self.ticker.clone();
        let base_url = self.config.base_url.clone();

        pb.enable_steady_tick(PROGRESS_UPDATE_INTERVAL);

        let worker_rx = shutdown_rx.resubscribe();
        let results = self.results.clone();
        let handles = self.spawn_workers(workers, stealers.clone(), worker_rx)?;

        // Wait for either completion or shutdown signal
        tokio::select! {
            res = async {
                for handle in handles {
                    handle.await??;
                }
                Ok::<(), crate::error::RwalkError>(())
            } => {
                res?;
                pb.finish_and_clear();
                Ok((results, ticker.get_rate()))
            }
            e = shutdown_rx.recv() => {
                if matches!(e, Ok(true)) {
                    // Save state before returning
                    Self::save_state("rwalk.state", global_.clone(), results.clone(), base_url)?;
                }
                pb.finish_and_clear();
                Ok((results, ticker.get_rate()))
            }
        }
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
        shutdown_rx: broadcast::Receiver<bool>,
    ) -> Result<Vec<JoinHandle<Result<()>>>> {
        let self_ = Arc::new(self); // prevents us from cloning the entire struct for each worker
        workers
            .into_iter()
            .map(|worker| {
                let stealers = stealers.clone(); // Arc
                let results = self_.results.clone(); // Arc
                let self_ = self_.clone(); // Arc
                let shutdown_rx = shutdown_rx.resubscribe();

                let handle = tokio::spawn(async move {
                    self_.worker(worker, &stealers, results, shutdown_rx).await
                });

                Ok(handle)
            })
            .collect::<Result<Vec<_>>>()
    }

    async fn worker(
        &self,
        worker: Worker<Task>,
        stealers: &Vec<Stealer<Task>>,
        results: Arc<HashMap<String, RwalkResponse>>,
        mut shutdown_rx: broadcast::Receiver<bool>,
    ) -> Result<()> {
        while let Some(task) = utils::find_task(&worker, &self.global_queue, &stealers) {
            tokio::select! {
                res = async {
                    if let Some(ref throttler) = self.worker_config.throttler {
                        throttler.wait_for_request().await;
                    }

                    let response = self.process_request(&task).await?;
                    self.ticker.tick();
                    self.pb.inc(1);
                    if self.config.retry_codes.iter().any(|e| e.contains(response.status)) {
                        if task.retry < self.config.retries {
                            let mut task = task.clone();
                            task.retry();
                            self.global_queue.push(task);
                            self.pb.set_length(self.pb.length().unwrap() + 1);
                        } else {
                            self.pb.println(format!(
                                "{} Failed to fetch {} after {} retries ({})",
                                WARNING.yellow(),
                                task.url.bold(),
                                self.config.retries.yellow(),
                                response.status.dimmed()
                            ));
                        }

                        return Ok::<(), crate::error::RwalkError>(());
                    }
                    if self.worker_config.filterer.filter(&response)? {
                        self.worker_config.handler.handle(response.clone(), &self)?;
                        if self.config.bell {
                            bell();
                        }
                        results.insert(response.url.to_string(), response);
                    }

                    Ok::<(), crate::error::RwalkError>(())
                } => {
                    res?;
                }

                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_request(&self, task: &Task) -> Result<RwalkResponse> {
        let start = std::time::Instant::now();
        let res = self
            .worker_config
            .client
            .request(self.config.method.clone(), task.url.clone())
            .send()
            .await;
        match res {
            Ok(res) => {
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
                if task.retry < self.config.retries {
                    let mut task = task.clone();
                    task.retry();
                    self.global_queue.push(task);
                    self.pb.set_length(self.pb.length().unwrap() + 1);
                } else {
                    self.pb.println(format!(
                        "{} Failed to fetch {} after {} retries ({})",
                        WARNING.yellow(),
                        task.url.bold(),
                        self.config.retries.yellow(),
                        e
                    ));
                }

                let res = RwalkResponse::from_error(e, task.url.clone().parse()?, task.depth);
                Ok(res)
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
