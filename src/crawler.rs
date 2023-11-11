use crate::log::log_warning;

use super::ARGS;
use anyhow::{Context, Error, Result};
use async_recursion::async_recursion;
use colored::Colorize;
use reqwest::redirect::Policy;
use std::time::Duration;
use url::Url;

pub struct Crawler {
    client: reqwest::Client,
}

pub struct CrawlResult {
    pub urls: Vec<Url>,
    pub status_code: reqwest::StatusCode,
}

impl Crawler {
    pub fn new() -> Crawler {
        Crawler {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(ARGS.timeout))
                .user_agent(ARGS.user_agent.clone().unwrap_or_else(|| {
                    format!(
                        "Mozilla/5.0 (compatible; rwalk/{})",
                        env!("CARGO_PKG_VERSION"),
                    )
                }))
                .redirect(Policy::none())
                .build()
                .unwrap(),
        }
    }
    #[async_recursion]
    pub async fn crawl(&self, url: &Url, redirected: Vec<Url>, retries: u8) -> Result<CrawlResult> {
        let res = match ARGS.method.as_str() {
            "GET" => self
                .client
                .get(url.clone())
                .send()
                .await
                .with_context(|| format!("Failed to crawl {}", url.as_str())),
            "POST" => self
                .client
                .post(url.clone())
                .body(ARGS.data.clone().unwrap_or_default())
                .send()
                .await
                .with_context(|| format!("Failed to crawl {}", url.as_str())),
            "PUT" => self
                .client
                .put(url.clone())
                .body(ARGS.data.clone().unwrap_or_default())
                .send()
                .await
                .with_context(|| format!("Failed to crawl {}", url.as_str())),
            "DELETE" => self
                .client
                .delete(url.clone())
                .body(ARGS.data.clone().unwrap_or_default())
                .send()
                .await
                .with_context(|| format!("Failed to crawl {}", url.as_str())),
            "HEAD" => self
                .client
                .head(url.clone())
                .body(ARGS.data.clone().unwrap_or_default())
                .send()
                .await
                .with_context(|| format!("Failed to crawl {}", url.as_str())),
            "OPTIONS" => self
                .client
                .request(reqwest::Method::OPTIONS, url.clone())
                .body(ARGS.data.clone().unwrap_or_default())
                .send()
                .await
                .with_context(|| format!("Failed to crawl {}", url.as_str())),
            "CONNECT" => self
                .client
                .request(reqwest::Method::CONNECT, url.clone())
                .body(ARGS.data.clone().unwrap_or_default())
                .send()
                .await
                .with_context(|| format!("Failed to crawl {}", url.as_str())),
            "TRACE" => self
                .client
                .request(reqwest::Method::TRACE, url.clone())
                .body(ARGS.data.clone().unwrap_or_default())
                .send()
                .await
                .with_context(|| format!("Failed to crawl {}", url.as_str())),
            "PATCH" => self
                .client
                .request(reqwest::Method::PATCH, url.clone())
                .body(ARGS.data.clone().unwrap_or_default())
                .send()
                .await
                .with_context(|| format!("Failed to crawl {}", url.as_str())),
            _ => {
                log_warning(
                    &format!(
                        "{} {}",
                        url.as_str(),
                        format!("Invalid method, using GET").yellow()
                    ),
                    true,
                );
                self.client
                    .get(url.clone())
                    .send()
                    .await
                    .with_context(|| format!("Failed to crawl {}", url.as_str()))
            }
        };

        if let std::result::Result::Err(err) = res {
            if ARGS.retries > retries {
                // Retry
                log_warning(
                    &format!(
                        "{} {} {}",
                        url.as_str(),
                        err.to_string().red(),
                        format!("Retrying ({})", retries + 1).yellow()
                    ),
                    true,
                );
                return self.crawl(url, redirected, retries + 1).await;
            } else {
                // Failed
                return Err(Error::msg(format!(
                    "{} {}",
                    url.as_str(),
                    format!("Failed ({})", err).red()
                )));
            }
        }

        let res = res.unwrap();

        // If location is relative, make it absolute
        let mut redirected = redirected.clone();
        redirected.push(url.clone());
        match res.status().as_u16() {
            // Success
            200..=299 => Ok(CrawlResult {
                urls: redirected,
                status_code: res.status(),
            }),
            // Redirect
            300..=399 => {
                if redirected.len() < ARGS.redirects as usize {
                    // Follow redirect
                    let location = res
                        .headers()
                        .get("location")
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    if Url::parse(&location).is_err() {
                        let mut new_location = url.clone();
                        new_location.set_path(&location);
                        new_location.set_query(None);
                        new_location.set_fragment(None);

                        self.crawl(&new_location, redirected, retries).await
                    } else {
                        self.crawl(&Url::parse(&location)?, redirected, retries)
                            .await
                    }
                } else {
                    // Too many redirects
                    log_warning(
                        &format!(
                            "{} {}",
                            url.as_str(),
                            format!("Too many redirects ({}), leaving as is", redirected.len())
                                .yellow()
                        ),
                        true,
                    );
                    Ok(CrawlResult {
                        urls: redirected,
                        status_code: res.status(),
                    })
                }
            }
            _ => {
                // Failed
                Err(Error::msg(format!(
                    "{} {}",
                    url.as_str(),
                    format!("Failed ({})", res.status().as_u16()).red()
                )))
            }
        }
    }
}
