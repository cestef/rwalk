use std::{
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::Mutex,
    time::{sleep, Instant as TokioInstant},
};

pub struct DynamicThrottler {
    // Current requests per second limit
    current_rps: AtomicU64,
    // Minimum allowed RPS
    min_rps: u64,
    // Maximum allowed RPS
    max_rps: u64,
    // Number of recent errors
    error_count: AtomicUsize,
    // Number of recent successful requests
    success_count: AtomicUsize,
    // Last adjustment time
    last_adjustment: Arc<Mutex<Instant>>,
    // Window size for error rate calculation (in seconds)
    window_size: Duration,
    // Error threshold percentage for throttling
    error_threshold: f64,
    // Token bucket for rate limiting
    last_request: Arc<Mutex<TokioInstant>>,
    // Number of worker threads
    worker_count: usize,
}

impl DynamicThrottler {
    pub fn new(
        min_rps: u64,
        max_rps: u64,
        worker_count: usize,
        window_size_millis: u64,
        error_treshold: f64,
    ) -> Self {
        Self {
            current_rps: AtomicU64::new(max_rps),
            min_rps,
            max_rps,
            error_count: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            last_adjustment: Arc::new(Mutex::new(Instant::now())),
            window_size: Duration::from_millis(window_size_millis),
            error_threshold: error_treshold,
            last_request: Arc::new(Mutex::new(TokioInstant::now())),
            worker_count,
        }
    }

    pub fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Wait for the appropriate delay before the next request
    pub async fn wait_for_request(&self) {
        let current_rps = self.current_rps.load(Ordering::Relaxed);
        let delay = Duration::from_secs_f64(1.0 / current_rps as f64);

        let mut last = self.last_request.lock().await;
        let now = TokioInstant::now();
        let elapsed = now.duration_since(*last);

        if elapsed < delay {
            sleep(delay - elapsed).await;
        }

        *last = TokioInstant::now();

        // Try to adjust the rate after updating last_request
        self.adjust_rate().await;
    }

    async fn adjust_rate(&self) {
        let mut last_adjustment = self.last_adjustment.lock().await;
        let now = Instant::now();

        if now.duration_since(*last_adjustment) < self.window_size {
            return;
        }

        let errors = self.error_count.swap(0, Ordering::Relaxed);
        let successes = self.success_count.swap(0, Ordering::Relaxed);
        let total = errors + successes;

        if total == 0 {
            return;
        }

        let error_rate = errors as f64 / total as f64;
        let current = self.current_rps.load(Ordering::Relaxed);

        let new_rps = if error_rate > self.error_threshold {
            (current as f64 * 0.8) as u64
        } else if error_rate < self.error_threshold / 2.0 {
            (current as f64 * 1.1) as u64
        } else {
            current
        };

        let new_rps = new_rps.clamp(self.min_rps, self.max_rps);

        if new_rps != current {
            self.current_rps.store(new_rps, Ordering::Relaxed);
            println!(
                "Throttling adjusted: {} → {} RPS (error rate: {:.2}%, {} workers)",
                current,
                new_rps,
                error_rate * 100.0,
                self.worker_count
            );
        }

        *last_adjustment = now;
    }

    pub fn get_current_rps(&self) -> u64 {
        self.current_rps.load(Ordering::Relaxed)
    }
}
