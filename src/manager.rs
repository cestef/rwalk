use std::sync::{Arc, Mutex};

use anyhow::Result;
use colored::Colorize;
use indicatif::ProgressBar;
use stopwatch::Stopwatch;
use url::Url;

use crate::{
    crawler,
    log::{log_error, log_success, log_verbose},
    ARGS,
};

pub struct CrawlerManager {
    pub url: Url,
    pub words: Vec<String>,
    pub threads: usize,
    progress: ProgressBar,
}

impl CrawlerManager {
    pub fn new(
        url: Url,
        words: Vec<String>,
        threads: usize,
        progress: ProgressBar,
    ) -> CrawlerManager {
        CrawlerManager {
            url,
            threads,
            words,
            progress,
        }
    }

    pub async fn run(&self) -> Result<Vec<Vec<String>>> {
        let chunks = self.words.chunks(self.threads);
        let res: Arc<Mutex<Vec<Vec<String>>>> = Arc::new(Mutex::new(Vec::new()));
        let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();
        for chunk in chunks {
            let words = chunk.to_vec();
            let host = self.url.clone();
            let progress = self.progress.clone();
            let res = res.clone();
            handles.push(tokio::spawn(async move {
                let mut found: Vec<Vec<String>> = Vec::new();
                let crawler = crawler::Crawler::new();
                for word in &words {
                    let watch = Stopwatch::start_new();
                    let mut new_path = host.path().to_string();
                    let word_sanitized = word.replace("/", "");
                    // Remove trailing slash for new_path
                    if new_path.ends_with("/") {
                        new_path.pop();
                    }
                    new_path.push_str(format!("/{}", word_sanitized).as_str());
                    let mut url = host.clone();
                    url.set_path(&new_path);
                    let res = crawler.crawl(&url, vec![], 0).await;

                    match res {
                        std::result::Result::Ok(res) => {
                            found.push(res.urls.iter().map(|u| u.to_string()).collect());
                            if res.urls.len() > 1 {
                                log_success(
                                    &format!(
                                        "{} {} {}",
                                        res.status_code.as_u16().to_string().green(),
                                        format!(
                                            "{} → {}",
                                            res.urls.first().unwrap(),
                                            res.urls.last().unwrap()
                                        ),
                                        format!("({}ms)", watch.elapsed_ms()).dimmed()
                                    ),
                                    true,
                                );
                            } else {
                                log_success(
                                    &format!(
                                        "{} {} {}",
                                        res.status_code.as_u16().to_string().green(),
                                        res.urls.last().unwrap(),
                                        format!("({}ms)", watch.elapsed_ms()).dimmed()
                                    ),
                                    true,
                                );
                            }
                        }
                        Err(err) => {
                            log_verbose(&format!("{} {} ({})", "✖".red(), word, err), true);
                        }
                    };
                    progress.inc(1);
                }

                if ARGS.throttle > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(ARGS.throttle)).await;
                }

                // Push the found paths to the result
                res.lock().unwrap().extend(found);
            }));
        }

        // Wait for all threads to finish
        for handle in handles {
            handle.await?;
        }

        // Clone the result to avoid locking the mutex
        let found = res.lock().unwrap().clone();
        Ok(found)
    }
    pub fn set_url(&mut self, url: Url) {
        self.url = url;
    }
}
