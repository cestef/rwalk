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
    current_rps: AtomicU64,
    min_rps: u64,
    max_rps: u64,
    error_count: AtomicUsize,
    success_count: AtomicUsize,
    last_adjustment: Arc<Mutex<Instant>>,
    window_size: Duration,
    error_threshold: f64,
    request_counter: AtomicU64,
    window_start: Arc<Mutex<TokioInstant>>,
    last_error_rate: Arc<Mutex<f64>>,
    high_watermark: AtomicU64, // Highest successful RPS
    low_watermark: AtomicU64,  // Lowest failed RPS
    phase: Arc<Mutex<SearchPhase>>,
}

#[derive(Debug, Clone, PartialEq)]
enum SearchPhase {
    Discovery,    // Initial phase to find upper bound
    BinarySearch, // Binary search between bounds
    Stabilize,    // Fine-tune around found optimum
}

impl DynamicThrottler {
    pub fn new(min_rps: u64, max_rps: u64, window_size_millis: u64, error_threshold: f64) -> Self {
        let start_rps = min_rps;
        Self {
            current_rps: AtomicU64::new(start_rps),
            min_rps: min_rps.max(1),
            max_rps,
            error_count: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            last_adjustment: Arc::new(Mutex::new(Instant::now())),
            window_size: Duration::from_millis(window_size_millis),
            error_threshold,
            request_counter: AtomicU64::new(0),
            window_start: Arc::new(Mutex::new(TokioInstant::now())),
            last_error_rate: Arc::new(Mutex::new(0.0)),
            high_watermark: AtomicU64::new(min_rps),
            low_watermark: AtomicU64::new(max_rps),
            phase: Arc::new(Mutex::new(SearchPhase::Discovery)),
        }
    }

    pub fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn wait_for_request(&self) {
        let current_rps = self.current_rps.load(Ordering::Relaxed);
        let rps = current_rps.max(1);

        // Calculate the theoretical time between requests
        let interval = Duration::from_secs_f64(1.0 / rps as f64);

        // Get our position in the current window
        let request_number = self.request_counter.fetch_add(1, Ordering::Relaxed);

        // Calculate when this request should occur
        let target_time = {
            let mut window_start = self.window_start.lock().await;
            let now = TokioInstant::now();

            // If we've passed our window, start a new one
            if now.duration_since(*window_start) >= Duration::from_secs(2) {
                *window_start = now;
                self.request_counter.store(1, Ordering::Relaxed);
            }

            *window_start + interval * request_number as u32
        };

        // Wait until our target time
        let now = TokioInstant::now();
        if target_time > now {
            sleep(target_time - now).await;
        }

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
        let mut phase = self.phase.lock().await;
        let mut last_error_rate = self.last_error_rate.lock().await;

        let new_rps = match *phase {
            SearchPhase::Discovery => {
                if error_rate > self.error_threshold || current >= self.max_rps {
                    // Found upper bound, switch to binary search
                    self.low_watermark.store(
                        self.high_watermark.load(Ordering::Relaxed),
                        Ordering::Relaxed,
                    );
                    self.high_watermark.store(current, Ordering::Relaxed);
                    *phase = SearchPhase::BinarySearch;
                    (self.high_watermark.load(Ordering::Relaxed)
                        + self.low_watermark.load(Ordering::Relaxed))
                        / 2
                } else {
                    // Keep increasing exponentially
                    self.high_watermark.store(current, Ordering::Relaxed);
                    current * 2
                }
            }
            SearchPhase::BinarySearch => {
                if error_rate > self.error_threshold {
                    self.high_watermark.store(current, Ordering::Relaxed);
                } else {
                    self.low_watermark.store(current, Ordering::Relaxed);
                }

                let new = (self.high_watermark.load(Ordering::Relaxed)
                    + self.low_watermark.load(Ordering::Relaxed))
                    / 2;
                if (self.high_watermark.load(Ordering::Relaxed)
                    - self.low_watermark.load(Ordering::Relaxed))
                    <= 2
                {
                    *phase = SearchPhase::Stabilize;
                }
                new
            }
            SearchPhase::Stabilize => {
                if error_rate > self.error_threshold {
                    current - 1
                } else if *last_error_rate == 0.0 && error_rate == 0.0 {
                    current + 1
                } else {
                    current
                }
            }
        };

        let new_rps = new_rps.clamp(self.min_rps, self.max_rps);
        *last_error_rate = error_rate;

        if new_rps != current {
            self.current_rps.store(new_rps, Ordering::Relaxed);
            println!(
                "Throttling adjusted: {} → {} RPS (error rate: {:.2}%, phase: {:?}, HWM: {}, LWM: {})",
                current,
                new_rps,
                error_rate * 100.0,
                *phase,
                self.high_watermark.load(Ordering::Relaxed),
                self.low_watermark.load(Ordering::Relaxed)
            );
        } else {
            println!(
                "Throttling stable: {} RPS (error rate: {:.2}%, phase: {:?}, HWM: {}, LWM: {})",
                current,
                error_rate * 100.0,
                *phase,
                self.high_watermark.load(Ordering::Relaxed),
                self.low_watermark.load(Ordering::Relaxed)
            );
        }

        *last_adjustment = now;
    }

    pub fn get_current_rps(&self) -> u64 {
        self.current_rps.load(Ordering::Relaxed)
    }
}
