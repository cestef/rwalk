use std::{sync::Arc, time::Duration};
use tokio::{
    sync::Semaphore,
    time::{sleep, Instant},
};

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

    pub async fn wait_for_request(&self) {
        self.semaphore.acquire().await.unwrap().forget();
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
