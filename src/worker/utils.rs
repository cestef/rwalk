use crate::Result;
use crossbeam::deque::{Injector, Stealer, Worker};
use dashmap::DashMap as HashMap;
use reqwest::StatusCode;
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
pub struct RwalkResponse {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub url: reqwest::Url,
    pub time: std::time::Duration,
    pub depth: usize,
}

impl RwalkResponse {
    pub async fn from_response(
        response: reqwest::Response,
        parse_body: bool,
        start: std::time::Instant,
        depth: usize,
    ) -> Result<Self> {
        let status = response.status();
        let url = response.url().clone();
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
            url,
            time: start.elapsed(),
            depth,
        })
    }
}

pub fn is_directory(response: &RwalkResponse) -> bool {
    if let Some(content_type) = response.headers.get(reqwest::header::CONTENT_TYPE.as_str()) {
        if let Some(body) = &response.body {
            if content_type.starts_with("text/html") {
                if is_html_directory(body) {
                    // println!("{} is HTML", response.url);

                    return true;
                }
            }
        }
    }
    if response.status.is_redirection() {
        // status code is 3xx
        match response.headers.get(reqwest::header::LOCATION.as_str()) {
            // and has a Location header
            Some(loc) => {
                // get absolute redirect Url based on the already known base url
                // println!("Location header: {:?}", loc);

                if let Ok(abs_url) = response.url.join(&loc) {
                    if format!("{}/", response.url) == abs_url.as_str() {
                        // if current response's Url + / == the absolute redirection
                        // location, we've found a directory suitable for recursion
                        // println!("found directory suitable for recursion: {}", response.url);
                        return true;
                    }
                }
            }
            None => {
                // println!(
                //     "expected Location header, but none was found: {:?}",
                //     response
                // );
                return false;
            }
        }
    } else if response.status.is_success()
        || matches!(
            response.status,
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED // 403, 401 ; a little bit of a hack but it works most of the time
        )
    {
        // status code is 2xx or 403, need to check if it ends in /

        if response.url.as_str().ends_with('/') {
            // println!("{} is directory suitable for recursion", response.url);
            return true;
        } else {
            // println!("{} is not a directory", response.url);
            return false;
        }
    }

    false
}

const HTML_DIRECTORY_INDICATORS: [&str; 4] = [
    "index of",
    "nginx directory listing",
    "directory listing -- /",
    "directory listing for /",
];

pub fn is_html_directory(body: &str) -> bool {
    return HTML_DIRECTORY_INDICATORS
        .iter()
        .any(|&indicator| body.contains(&indicator.to_lowercase()));
}
