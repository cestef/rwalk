pub mod handler;
pub mod pool;

use std::collections::HashMap;

pub use pool::WorkerPool;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub url: String,
    pub depth: usize,
    pub data: Option<String>,
    pub headers: HashMap<String, String>,
    pub retry: usize,
}

impl Task {
    pub fn new_recursive(url: String, depth: usize) -> Self {
        Self {
            url,
            depth,
            data: None,
            headers: HashMap::new(),
            retry: 0,
        }
    }

    pub fn new_template(
        url: String,
        data: Option<String>,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            url,
            data,
            headers,
            retry: 0,
            depth: 0,
        }
    }

    fn retry(&mut self) {
        self.retry += 1;
    }
}
