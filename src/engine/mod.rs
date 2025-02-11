pub mod handler;
pub mod pool;

pub use pool::WorkerPool;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub url: String,
    pub depth: usize,
    pub retry: usize,
}

impl Task {
    pub fn new(url: String, depth: usize) -> Self {
        Self {
            url,
            depth,
            retry: 0,
        }
    }

    fn retry(&mut self) {
        self.retry += 1;
    }
}
