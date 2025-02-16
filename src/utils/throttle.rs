use std::{sync::Arc, time::Duration};
use tokio::{sync::Semaphore, time::sleep};

pub struct SimpleThrottler {
    rps: u64,
    semaphore: Arc<Semaphore>,
}

impl SimpleThrottler {
    pub fn new(rps: u64) -> Self {
        let instance = Self {
            rps,
            semaphore: Arc::new(Semaphore::new(rps as usize)),
        };
        // Spawn token replenishment task
        let sem = instance.semaphore.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;
                sem.add_permits(rps as usize);
            }
        });

        instance
    }

    pub async fn wait_for_request(&self) {
        self.semaphore.acquire().await.unwrap().forget();
    }
}
