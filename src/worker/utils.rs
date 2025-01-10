use crate::Result;
use crossbeam::deque::{Injector, Stealer, Worker};
use std::iter;

pub fn find_task<T>(local: &Worker<T>, global: &Injector<T>, stealers: &[Stealer<T>]) -> Option<T> {
    // Pop a task from the local queue, if not empty.
    local.pop().or_else(|| {
        // Otherwise, we need to look for a task elsewhere.
        iter::repeat_with(|| {
            // Try stealing a batch of tasks from the global queue.
            global
                .steal_batch_and_pop(local)
                // Or try stealing a task from one of the other threads.
                .or_else(|| stealers.iter().map(|s| s.steal()).collect())
        })
        // Loop while no task was stolen and any steal operation needs to be retried.
        .find(|s| !s.is_retry())
        // Extract the stolen task, if there is one.
        .and_then(|s| s.success())
    })
}

#[derive(Debug, Clone)]
pub struct SendableResponse {
    pub status: u16,
    pub headers: papaya::HashMap<String, String>,
    pub body: Option<String>,
}

impl SendableResponse {
    pub async fn from_response(response: reqwest::Response, parse_body: bool) -> Result<Self> {
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(k, v)| Some((k.as_str().to_string(), v.to_str().ok()?.to_string())))
            .collect();

        let body = if parse_body {
            Some(response.text().await?)
        } else {
            None
        };

        Ok(Self {
            status,
            headers,
            body,
        })
    }
}
