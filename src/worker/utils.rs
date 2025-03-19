use crate::{
    utils::{constants::STEAL_BATCH_LIMIT, directory},
    Result, RwalkError,
};
use crossbeam::deque::{Injector, Stealer, Worker};
use dashmap::DashMap as HashMap;
use rhai::{CustomType, TypeBuilder};

use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    iter,
    str::FromStr,
};

pub fn find_task<T>(local: &Worker<T>, global: &Injector<T>, stealers: &[Stealer<T>]) -> Option<T> {
    // Pop a task from the local queue, if not empty.
    local.pop().or_else(|| {
        // Otherwise, we need to look for a task elsewhere.
        iter::repeat_with(|| {
            // Try stealing a batch of tasks from the global queue.
            global
                .steal_batch_with_limit_and_pop(local, STEAL_BATCH_LIMIT)
                // Or try stealing a task from one of the other threads.
                .or_else(|| stealers.iter().map(|s| s.steal()).collect())
        })
        // Loop while no task was stolen and any steal operation needs to be retried.
        .find(|s| !s.is_retry())
        // Extract the stolen task, if there is one.
        .and_then(|s| s.success())
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, CustomType)]
pub struct RwalkResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub url: url::Url,
    pub time: std::time::Duration,
    pub depth: usize,
    pub r#type: ResponseType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseType {
    Directory,
    File(Option<String>),
    Error,
}

impl FromStr for ResponseType {
    type Err = RwalkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "directory" | "dir" | "d" => Ok(ResponseType::Directory),
            "error" | "e" => Ok(ResponseType::Error),
            e => Ok(ResponseType::File(Some(e.to_string()))),
        }
    }
}

impl Display for ResponseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResponseType::Directory => write!(f, "directory"),
            ResponseType::File(t) => write!(f, "file:{}", t.as_deref().unwrap_or("")),
            ResponseType::Error => write!(f, "error"),
        }
    }
}

impl RwalkResponse {
    pub async fn from_response(
        response: reqwest::Response,
        parse_body: bool,
        start: std::time::Instant,
        depth: usize,
    ) -> Result<Self> {
        let status = response.status().as_u16();
        let url = response.url().clone();
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| Some((k.as_str().to_string(), v.to_str().ok()?.to_string())))
            .collect();

        let file_type = headers
            .get("content-type")
            .map(|s| s.splitn(2, ';').next().unwrap().to_string());

        let body = if parse_body {
            Some(response.text().await?)
        } else {
            None
        };

        let mut res = Self {
            status,
            headers,
            body,
            url,
            time: start.elapsed(),
            depth,
            r#type: ResponseType::File(file_type),
        };

        if directory::check(&res) {
            res.r#type = ResponseType::Directory;
        }

        Ok(res)
    }

    pub fn from_error(e: reqwest::Error, url: url::Url, depth: usize) -> Self {
        let status = e.status().map_or(0, |s| s.as_u16());
        let headers = HashMap::new();
        let body = Some(e.to_string());
        let time = std::time::Duration::default();
        let r#type = ResponseType::Error;

        Self {
            status,
            headers,
            body,
            url,
            time,
            depth,
            r#type,
        }
    }
}
