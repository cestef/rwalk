use std::{
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex, Semaphore},
    time::{sleep, Instant as TokioInstant},
};

use crate::utils::constants::THROUGHPUT_THRESHOLD;

pub struct DynamicThrottler {
    current_rps: Arc<AtomicU64>,
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
    semaphore: Arc<Semaphore>,
    last_semaphore_refill: Arc<Mutex<Instant>>,
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
        let instance = Self {
            current_rps: Arc::new(AtomicU64::new(start_rps)),
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
            semaphore: Arc::new(Semaphore::new(start_rps as usize)),
            last_semaphore_refill: Arc::new(Mutex::new(Instant::now())),
        };
        // Spawn token replenishment task
        let sem = instance.semaphore.clone();
        let current_rps = instance.current_rps.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;
                let rps = current_rps.load(Ordering::Relaxed);
                sem.add_permits(rps as usize);
            }
        });

        instance
    }

    pub fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    async fn replenish_semaphore(&self) {
        let mut last_refill = self.last_semaphore_refill.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= Duration::from_secs(1) {
            let periods = elapsed.as_secs();
            if periods > 0 {
                let rps = self.current_rps.load(Ordering::Relaxed);
                self.semaphore.add_permits((rps * periods) as usize);
                *last_refill = now - Duration::from_nanos(elapsed.subsec_nanos() as u64);
            }
        }
    }

    pub async fn wait_for_request(&self) {
        self.replenish_semaphore().await;
        self.semaphore.acquire().await.unwrap().forget();
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

        let actual_rps = total as f64 / self.window_size.as_secs_f64();
        let error_rate = errors as f64 / total as f64;
        let current = self.current_rps.load(Ordering::Relaxed);
        let mut phase = self.phase.lock().await;
        let mut last_error_rate = self.last_error_rate.lock().await;

        let new_rps = match *phase {
            SearchPhase::Discovery => {
                if error_rate > self.error_threshold {
                    // Found error-based upper bound, switch to binary search
                    self.low_watermark.store(
                        self.high_watermark.load(Ordering::Relaxed),
                        Ordering::Relaxed,
                    );
                    self.high_watermark.store(current, Ordering::Relaxed);
                    *phase = SearchPhase::BinarySearch;
                    println!("Switching to binary search due to high error rate");
                    (self.high_watermark.load(Ordering::Relaxed)
                        + self.low_watermark.load(Ordering::Relaxed))
                        / 2
                } else if actual_rps < (current as f64 * THROUGHPUT_THRESHOLD) {
                    // Not achieving enough throughput for consecutive windows
                    // Switch to binary search with current rate as high watermark
                    self.high_watermark.store(current, Ordering::Relaxed);
                    self.low_watermark.store(self.min_rps, Ordering::Relaxed);
                    *phase = SearchPhase::BinarySearch;
                    println!("Switching to binary search due to throughput underperformance");
                    (current + self.min_rps) / 2
                } else if current >= self.max_rps {
                    // Hit max RPS limit
                    println!("Throttling at max RPS: {}", current);
                    *phase = SearchPhase::Stabilize;
                    current
                } else {
                    println!("Increasing RPS: {}", current);
                    self.high_watermark.store(current, Ordering::Relaxed);
                    (current * 2).min(self.max_rps)
                }
            }
            SearchPhase::BinarySearch => {
                if error_rate > self.error_threshold {
                    self.high_watermark.store(current, Ordering::Relaxed);
                } else if actual_rps >= (current as f64 * THROUGHPUT_THRESHOLD) {
                    self.low_watermark.store(current, Ordering::Relaxed);
                } else {
                    // Not achieving enough throughput, reduce target
                    self.high_watermark.store(current, Ordering::Relaxed);
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
                } else if actual_rps < (current as f64 * THROUGHPUT_THRESHOLD) {
                    current - 1
                } else if *last_error_rate == 0.0 && error_rate == 0.0 {
                    (current + 1).min(self.max_rps)
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
                "Throttling adjusted: {} → {} RPS (error rate: {:.2}%, phase: {:?}, HWM: {}, LWM: {}, actual: {:.2})",
                current,
                new_rps,
                error_rate * 100.0,
                *phase,
                self.high_watermark.load(Ordering::Relaxed),
                self.low_watermark.load(Ordering::Relaxed),
                actual_rps
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
