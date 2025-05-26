use crate::{
    RwalkError,
    cli::Opts,
    error::Result,
    filters::Filterer,
    types::EngineMode,
    utils::{
        bell,
        constants::{
            DEFAULT_RESPONSE_FILTER, PROGRESS_CHARS, PROGRESS_TEMPLATE, PROGRESS_UPDATE_INTERVAL,
            STATE_FILE,
        },
        format::WARNING,
        throttle::{MetricsThrottler, SimpleThrottler, Throttler, create_dynamic_throttler},
        ticker::RequestTickerNoReset,
        types::{IntRange, ThrottleMode},
    },
    wordlist::Wordlist,
    worker::{
        filters::ResponseFilterRegistry,
        utils::{self, RwalkResponse},
    },
};

use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use dashmap::DashMap;
use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use reqwest::{
    Client,
    header::{HeaderName, HeaderValue},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Write},
    path::Path,
    sync::Arc,
};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::debug;
use url::Url;

use super::{
    Task,
    handler::{ResponseHandler, recursive::RecursiveHandler, template::TemplateHandler},
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
    pub headers: Option<HashMap<usize, HashMap<String, String>>>,
    pub throttle_mode: ThrottleMode,
    pub data: Option<String>,
}

// Worker configuration
#[derive(Clone)]
pub struct WorkerConfig {
    pub client: Client,
    pub filterer: Filterer<RwalkResponse>,
    pub handler: Arc<Box<dyn ResponseHandler>>,
    pub throttler: Option<Arc<MetricsThrottler>>,
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
    pub results: Arc<DashMap<String, RwalkResponse>>,
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
        results: Arc<DashMap<String, RwalkResponse>>,
        base_url: Url,
    ) -> Result<()> {
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

        let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);
        writer.write(&rmp_serde::to_vec(&state)?)?;
        Ok(())
    }

    pub fn load_state<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file =
            File::open(path).map_err(|e| crate::error!(source = e, "Failed to open state file"))?;
        let reader = BufReader::new(file);
        let state: WorkerState = rmp_serde::from_read(reader)
            .map_err(|e| crate::error!(source = e, "Failed to deserialize state file"))?;

        if state.base_url != self.config.base_url {
            return Err(crate::error!(
                "State file does not match the current base URL"
            ));
        }

        for task in state.pending_tasks {
            self.global_queue.push(task);
        }

        for (url, response) in state.completed_results {
            self.results.insert(url, response);
        }

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
        let throttler = match config.throttle_mode {
            ThrottleMode::Dynamic => Some(Arc::new(MetricsThrottler::new(
                create_dynamic_throttler(config.rps.unwrap_or(0) as f64),
            ))),
            ThrottleMode::Simple => Some(Arc::new(MetricsThrottler::new(SimpleThrottler::new(
                config.rps.unwrap_or(0),
            )))),
            ThrottleMode::None => None,
        };
        let worker_config = WorkerConfig {
            client,
            filterer: filterer.clone(),
            handler: Arc::new(handler),
            throttler,
            needs_body: filterer.needs_body()? || config.show.contains(&"body".to_string()), // Precompute if any filter needs body
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
                results: Arc::new(DashMap::new()),
            },
            shutdown_tx,
        ))
    }

    pub fn from_opts(
        opts: &Opts,
        wordlists: Vec<Wordlist>,
    ) -> Result<(Self, broadcast::Sender<bool>)> {
        let headers = if opts.headers.is_empty() {
            None
        } else {
            let mut headers: HashMap<usize, HashMap<String, String>> =
                HashMap::with_capacity(opts.depth + 1);
            for (depths, name, value) in opts.headers.iter() {
                if depths.is_empty() {
                    // Apply to all depths (0 to max)
                    for i in 0..=opts.depth {
                        headers
                            .entry(i)
                            .or_default()
                            .insert(name.clone(), value.clone());
                    }
                } else {
                    // Apply only to specified depths
                    for depth in depths.iter() {
                        let depth: usize = depth.parse()?;
                        headers
                            .entry(depth)
                            .or_default()
                            .insert(name.clone(), value.clone());
                    }
                }
            }

            Some(headers)
        };

        let config = PoolConfig {
            threads: opts.threads,
            base_url: opts
                .url
                .clone()
                .ok_or_else(|| crate::error!("No URL provided (This should never happen)"))?,
            mode: opts.mode,
            rps: opts.throttle.map(|e| e.0),
            force_recursion: opts.force_recursion,
            retries: opts.retries,
            retry_codes: opts.retry_codes.clone(),
            show: opts.show.clone(),
            max_depth: opts.depth.overflowing_sub(1).0,
            bell: opts.bell,
            method: opts.method.into(),
            headers,
            throttle_mode: opts.throttle.map(|e| e.1).unwrap_or(ThrottleMode::None),
            data: opts.data.clone(),
        };

        let global_queue = Arc::new(Injector::new());
        let client = Self::create_client(opts)?;
        let filterer = Self::create_filterer(opts)?;

        Self::new(config, global_queue, client, filterer, wordlists)
    }

    fn create_client(opts: &Opts) -> Result<Client> {
        let mut builder = Client::builder();
        if opts.http1 {
            builder = builder.http1_only();
            debug!("Using HTTP/1.1");
        }
        if opts.http2 {
            builder = builder.http2_prior_knowledge();
            debug!("Using HTTP/2");
        }
        if opts.http3 {
            builder = builder.http3_prior_knowledge();
            debug!("Using HTTP/3");
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
    ) -> Result<(Arc<DashMap<String, RwalkResponse>>, f64)> {
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
        let throttler = self.worker_config.throttler.clone();
        if let Some(throttler) = throttler {
            let pb = pb.clone();
            tokio::spawn(async move {
                loop {
                    let metrics = throttler.get_metrics().await;

                    pb.set_message(format!("avg. {:.2}", metrics.average_rps));
                }
            });
        }
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
                    Self::save_state(STATE_FILE, global_.clone(), results.clone(), base_url)?;
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
        stealers: &[Stealer<Task>],
        results: Arc<DashMap<String, RwalkResponse>>,
        mut shutdown_rx: broadcast::Receiver<bool>,
    ) -> Result<()> {
        while let Some(task) = utils::find_task(&worker, &self.global_queue, stealers) {
            tokio::select! {
                res = async {
                    if let Some(ref throttler) = self.worker_config.throttler {
                        throttler.wait_for_request().await;
                    }

                    let response = self.process_request(&task).await?;
                    if let Some(ref throttler) = self.worker_config.throttler {
                        throttler.record_response(&response).await?;
                    }
                    self.ticker.tick();
                    self.pb.inc(1);
                    if self.config.retry_codes.iter().any(|e| e.contains(response.status as u16)) {
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
                        self.worker_config.handler.handle(response.clone(), self)?;
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

        let mut req = self
            .worker_config
            .client
            .request(self.config.method.clone(), task.url.clone());
        if !task.headers.is_empty() {
            req = req.headers(
                task.headers
                    .iter()
                    .map(|(k, v)| {
                        let name = HeaderName::from_bytes(k.as_bytes()).unwrap();
                        let value = HeaderValue::from_str(v).unwrap();
                        (name, value)
                    })
                    .collect(),
            );
        } else if let Some(headers) = self
            .config
            .headers
            .as_ref()
            .and_then(|h| h.get(&task.depth))
        {
            req = req.headers(
                headers
                    .iter()
                    .map(|(k, v)| {
                        let name = HeaderName::from_bytes(k.as_bytes()).unwrap();
                        let value = HeaderValue::from_str(v).unwrap();
                        (name, value)
                    })
                    .collect(),
            );
        }

        if let Some(ref data) = task.data {
            req = req.body(data.clone());
        } else if let Some(ref default_data) = self.config.data {
            req = req.body(default_data.clone());
        }

        let res = req.send().await;

        match res {
            Ok(res) => {
                RwalkResponse::from_response(res, self.worker_config.needs_body, start, task.depth)
                    .await
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
