use std::{collections::VecDeque, sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, RwLock, Semaphore},
    time::{Instant, sleep},
};

use crate::Result;
use crate::worker::utils::RwalkResponse;

pub struct SimpleThrottler {
    rps: u64,
    semaphore: Arc<Semaphore>,
}

impl SimpleThrottler {
    pub fn new(rps: u64) -> Self {
        let instance = Self {
            rps,
            semaphore: Arc::new(Semaphore::new(0)),
        };

        let sem = instance.semaphore.clone();
        tokio::spawn(async move {
            let interval = Duration::from_secs_f64(1.0 / rps as f64);
            let mut next_release = Instant::now();

            loop {
                let now = Instant::now();
                if next_release > now {
                    sleep(next_release - now).await;
                }

                sem.add_permits(1);

                next_release += interval;

                if Instant::now() > next_release + Duration::from_secs(1) {
                    next_release = Instant::now();
                }
            }
        });

        instance
    }
}

#[async_trait::async_trait]
impl Throttler for SimpleThrottler {
    async fn wait_for_request(&self) {
        self.semaphore.acquire().await.unwrap().forget();
    }
}

#[async_trait::async_trait]
pub trait Throttler: Send + Sync + 'static {
    async fn wait_for_request(&self);
    async fn record_response(&self, _request: &RwalkResponse) -> Result<()> {
        Ok(())
    }
}

use async_trait::async_trait;

pub struct DynamicThrottler {
    rate_limit: Arc<RwLock<f64>>, // current
    semaphore: Arc<Semaphore>,
    recent_responses: Arc<Mutex<Vec<(Instant, u16)>>>,
    config: ThrottlerConfig,
    consecutive_429s: Arc<RwLock<u32>>,
    last_adjustment: Arc<RwLock<Instant>>,
}

pub struct ThrottlerConfig {
    initial_rps: f64,
    max_rps: f64,
    min_rps: f64,
    // increase rate when successful
    increase_factor: f64,
    // decrease rate when rate limited
    decrease_factor: f64,
    window_size: Duration,
    // min time between adjustments
    adjustment_interval: Duration,
}

impl Default for ThrottlerConfig {
    fn default() -> Self {
        Self {
            initial_rps: 5.0,
            max_rps: 50.0,
            min_rps: 0.5,
            increase_factor: 1.1,
            decrease_factor: 0.75,
            window_size: Duration::from_secs(5),
            adjustment_interval: Duration::from_secs(1),
        }
    }
}

impl DynamicThrottler {
    pub fn new(config: ThrottlerConfig) -> Self {
        let rate_limit = Arc::new(RwLock::new(config.initial_rps));
        let semaphore = Arc::new(Semaphore::new(0));
        let recent_responses = Arc::new(Mutex::new(Vec::new()));
        let consecutive_429s = Arc::new(RwLock::new(0));
        let last_adjustment = Arc::new(RwLock::new(Instant::now()));

        let instance = Self {
            rate_limit,
            semaphore,
            recent_responses,
            config,
            consecutive_429s,
            last_adjustment,
        };

        // Start token release background task
        let sem = instance.semaphore.clone();
        let rate = instance.rate_limit.clone();
        tokio::spawn(async move {
            let mut next_release = Instant::now();

            loop {
                let current_rate = *rate.read().await;
                let interval = Duration::from_secs_f64(1.0 / current_rate);

                let now = Instant::now();
                if next_release > now {
                    sleep(next_release - now).await;
                }

                sem.add_permits(1);
                next_release += interval;

                // Reset timing if we're significantly behind
                if Instant::now() > next_release + Duration::from_secs(1) {
                    next_release = Instant::now();
                }
            }
        });

        instance
    }

    async fn adjust_rate_limit(&self) {
        let now = Instant::now();
        let mut last_adj = self.last_adjustment.write().await;

        if now - *last_adj < self.config.adjustment_interval {
            return;
        }
        *last_adj = now;

        let mut responses = self.recent_responses.lock().await;
        let window_start = now - self.config.window_size;

        // remove old responses
        responses.retain(|(time, _)| *time >= window_start);

        let rate_limited_count = responses
            .iter()
            .filter(|(_, status)| *status == 429)
            .count();

        let mut current_rate = self.rate_limit.write().await;
        let cons_429s = self.consecutive_429s.read().await;

        if rate_limited_count > 2 || *cons_429s > 2 {
            // rate limited
            *current_rate = (*current_rate * self.config.decrease_factor).max(self.config.min_rps);

            // steeper reduction for consecutive 429s
            if *cons_429s > 3 {
                *current_rate =
                    (*current_rate * (0.9_f64.powi(*cons_429s as i32))).max(self.config.min_rps);
            }
        } else if !responses.is_empty() {
            // increase rate on success
            *current_rate = (*current_rate * self.config.increase_factor).min(self.config.max_rps);
        }

        // stay within bounds
        *current_rate = current_rate.clamp(self.config.min_rps, self.config.max_rps);

        responses.clear();
    }
}

#[async_trait]
impl Throttler for DynamicThrottler {
    async fn wait_for_request(&self) {
        self.semaphore.acquire().await.unwrap().forget();
    }

    async fn record_response(&self, response: &RwalkResponse) -> Result<()> {
        let status = response.status as u16;

        {
            let mut responses = self.recent_responses.lock().await;
            responses.push((Instant::now(), status));
        }

        {
            let mut cons_429s = self.consecutive_429s.write().await;
            if status == 429 {
                *cons_429s += 1;
            } else {
                *cons_429s = 0;
            }
        }

        self.adjust_rate_limit().await;

        Ok(())
    }
}

pub fn create_dynamic_throttler(initial_rps: f64) -> impl Throttler {
    let config = ThrottlerConfig {
        initial_rps,
        max_rps: initial_rps * 5.0,
        min_rps: initial_rps * 0.1,
        ..Default::default()
    };

    DynamicThrottler::new(config)
}
pub struct MetricsThrottler {
    inner: Arc<dyn Throttler>,
    metrics: Arc<RwLock<ThrottleMetricsInternal>>,
    start_time: Instant,
    recent_requests: Arc<Mutex<VecDeque<(Instant, u16)>>>,
    window_size: Duration,
}

#[derive(Debug, Clone, Default)]
struct ThrottleMetricsInternal {
    current_rps: f64,
    peak_rps: f64,
    total_requests: u64,
    total_429s: u64,
}

#[derive(Debug, Clone)]
pub struct ThrottleMetrics {
    pub current_rps: f64,
    pub peak_rps: f64,
    pub average_rps: f64,
    pub total_requests: u64,
    pub total_429s: u64,
    pub uptime_seconds: u64,
}

impl MetricsThrottler {
    pub fn new(inner: impl Throttler + 'static) -> Self {
        Self::with_config(inner, Duration::from_secs(10), 100)
    }

    pub fn with_config(
        inner: impl Throttler + 'static,
        window_size: Duration,
        max_recent_requests: usize,
    ) -> Self {
        let instance = Self {
            inner: Arc::new(inner),
            metrics: Arc::new(RwLock::new(ThrottleMetricsInternal::default())),
            start_time: Instant::now(),
            recent_requests: Arc::new(Mutex::new(VecDeque::with_capacity(max_recent_requests))),
            window_size,
        };

        // Start background task to periodically update RPS metrics
        let metrics_clone = instance.metrics.clone();
        let recent_requests_clone = instance.recent_requests.clone();
        let window_size_clone = instance.window_size;
        tokio::spawn(async move {
            let update_interval = Duration::from_millis(500);
            loop {
                sleep(update_interval).await;
                Self::update_rps_metric(&metrics_clone, &recent_requests_clone, window_size_clone)
                    .await;
            }
        });

        instance
    }

    pub async fn get_metrics(&self) -> ThrottleMetrics {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let metrics = self.metrics.read().await;

        ThrottleMetrics {
            current_rps: metrics.current_rps,
            peak_rps: metrics.peak_rps,
            average_rps: if elapsed > 0.0 {
                metrics.total_requests as f64 / elapsed
            } else {
                0.0
            },
            total_requests: metrics.total_requests,
            total_429s: metrics.total_429s,
            uptime_seconds: elapsed as u64,
        }
    }

    async fn update_rps_metric(
        metrics: &Arc<RwLock<ThrottleMetricsInternal>>,
        recent_requests: &Arc<Mutex<VecDeque<(Instant, u16)>>>,
        window_size: Duration,
    ) {
        let mut requests = recent_requests.lock().await;
        if requests.is_empty() {
            return;
        }

        let now = Instant::now();
        let window_start = now - window_size;

        // Remove expired requests
        while let Some((time, _)) = requests.front() {
            if *time < window_start {
                requests.pop_front();
            } else {
                break;
            }
        }

        let recent_count = requests.len();
        let current_rps = recent_count as f64 / window_size.as_secs_f64();

        // Update metrics
        let mut metrics_write = metrics.write().await;
        metrics_write.current_rps = current_rps;
        if current_rps > metrics_write.peak_rps {
            metrics_write.peak_rps = current_rps;
        }
    }
}

#[async_trait]
impl Throttler for MetricsThrottler {
    async fn wait_for_request(&self) {
        self.inner.wait_for_request().await;

        // Increment total requests counter
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
    }

    async fn record_response(&self, response: &RwalkResponse) -> Result<()> {
        let status = response.status as u16;

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            if status == 429 {
                metrics.total_429s += 1;
            }
        }

        // Update recent requests
        {
            let mut recent_requests = self.recent_requests.lock().await;
            recent_requests.push_back((Instant::now(), status));

            // This is now handled by the update_rps_metric function, but we'll maintain
            // the capacity limit here as a safeguard
            if recent_requests.len() > recent_requests.capacity() {
                recent_requests.pop_front();
            }
        }

        // Pass to inner throttler
        self.inner.record_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_smooth_throttler() {
        let throttler = Arc::new(SimpleThrottler::new(10)); // 10 RPS
        let start = Instant::now();
        let mut tasks = Vec::new();
        for _ in 0..10 {
            tasks.push(tokio::spawn({
                let throttler = throttler.clone();
                async move {
                    throttler.wait_for_request().await;
                }
            }));
        }

        for task in tasks {
            task.await.unwrap();
        }

        let elapsed = start.elapsed();
        println!("Elapsed: {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(1));
    }
}
