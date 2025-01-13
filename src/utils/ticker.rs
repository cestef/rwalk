use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub struct RequestTicker {
    // Counter for requests within the current window
    count: AtomicU64,
    // Last time the window was reset
    last_reset: RwLock<Instant>,
    // Window duration in seconds
    window_size: Duration,
}

impl RequestTicker {
    pub fn new(window_size_secs: u64) -> Arc<Self> {
        Arc::new(Self {
            count: AtomicU64::new(0),
            last_reset: RwLock::new(Instant::now()),
            window_size: Duration::from_secs(window_size_secs),
        })
    }

    pub fn tick(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_rate(&self) -> f64 {
        let now = Instant::now();
        let last_reset = *self.last_reset.read().unwrap();
        let elapsed = now.duration_since(last_reset);

        if elapsed >= self.window_size {
            // Reset the window if it has expired
            if let Ok(mut reset_guard) = self.last_reset.write() {
                // Double-check that another thread hasn't reset already
                if now.duration_since(*reset_guard) >= self.window_size {
                    *reset_guard = now;
                    let count = self.count.swap(0, Ordering::Relaxed);
                    return count as f64 / elapsed.as_secs_f64();
                }
            }
        }

        // If window hasn't expired, calculate current rate
        let count = self.count.load(Ordering::Relaxed);
        count as f64 / elapsed.as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_multithreaded_ticker() {
        let ticker = RequestTicker::new(1);
        let mut handles = vec![];

        // Spawn 10 threads that each increment the counter 1000 times
        for _ in 0..10 {
            let ticker_clone = ticker.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    ticker_clone.tick();
                    thread::sleep(Duration::from_micros(100));
                }
            }));
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let final_rate = ticker.get_rate();
        println!("Final rate: {} requests/sec", final_rate);
        assert!(final_rate > 0.0);
    }
}
