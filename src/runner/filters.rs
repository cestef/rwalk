use colored::Colorize;
use log::warn;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    cli::opts::Opts,
    utils::{
        check_range,
        constants::{DEFAULT_STATUS_CODES, ERROR, WARNING},
        parse_range_input,
    },
};

// Returns true if the response should be kept
pub fn check(
    opts: &Opts,
    res_text: &str,
    headers: &reqwest::header::HeaderMap,
    status_code: u16,
    time: u128,
    depth: Option<usize>,
) -> bool {
    let mut outs: Vec<bool> = Vec::new();

    // Default status filter
    let filters = if opts.filter.iter().any(|e| e.0 == "status") {
        opts.filter.clone()
    } else {
        let mut filters = opts.filter.clone();
        filters.push(("status".to_string(), DEFAULT_STATUS_CODES.to_string()));
        filters
    };

    for filter in filters {
        let mut filter = filter;
        // if the filter starts with [depth] then we parse the depth and remove it from the filter
        let filter_depth = if filter.0.starts_with('[') {
            let start_index = filter.0.find('[').unwrap();
            let end_index = filter.0.find(']').unwrap();
            let depth = filter.0[start_index + 1..end_index].parse::<usize>();
            filter.0 = filter.0[end_index + 1..].to_string();
            if let Ok(d) = depth {
                Some(d)
            } else {
                warn!("Invalid depth filter: {}", depth.unwrap_err());
                None
            }
        } else {
            None
        };

        // If this filter is not for the current depth, we skip it
        if filter_depth.is_some() && depth.is_none() {
            warn!("You provided a depth filter but you are not scanning recursively");
        }
        if filter_depth.is_some() && depth.is_some() && filter_depth != depth {
            continue;
        }
        let negated = filter.0.starts_with('!');
        let out = match filter.0.trim_start_matches('!') {
            "time" => check_range(&parse_range_input(&filter.1).unwrap(), time as usize) ^ negated,
            "status" => {
                check_range(&parse_range_input(&filter.1).unwrap(), status_code as usize) ^ negated
            }
            "contains" => !res_text.contains(&filter.1) ^ negated,
            "starts" => !res_text.starts_with(&filter.1) ^ negated,
            "ends" => !res_text.ends_with(&filter.1) ^ negated,
            "regex" => regex::Regex::new(&filter.1).unwrap().is_match(res_text) ^ negated,
            "length" | "size" => {
                check_range(&parse_range_input(&filter.1).unwrap(), res_text.len()) ^ negated
            }
            "hash" => filter.1.contains(&format!("{:x}", md5::compute(res_text))) ^ negated,
            "header" => {
                let mut header = filter.1.split('=');
                if let Some(key) = header.next() {
                    if let Some(value) = header.next() {
                        let header_value = headers.get(key);
                        (if let Some(header_value) = header_value {
                            header_value.to_str().unwrap() == value
                        } else {
                            false
                        }) ^ negated
                    } else {
                        warn!("Missing value in filter: {}", filter.1);
                        true
                    }
                } else {
                    warn!("Missing header key in filter: {}", filter.1);
                    true
                }
            }
            // json:jsonpath=value1|value2
            "json" => {
                if let Some(split_index) = filter.1.find('=') {
                    let (accessor, values) = filter.1.split_at(split_index);
                    let values = values.trim_start_matches('=');
                    let accessor = accessor.trim_end_matches('=');
                    let json: serde_json::Value = match serde_json::from_str(res_text) {
                        Ok(json) => json,
                        Err(e) => {
                            warn!("Response is not valid JSON: {}", e);
                            return true;
                        }
                    };
                    let json_value = accessor.split('.').fold(&json, |acc, x| {
                        acc.get(x).unwrap_or(&serde_json::Value::Null)
                    });
                    values.split('|').any(|value| {
                        json_value
                            .to_string()
                            .contains(value.trim_start_matches('!'))
                    }) ^ negated
                } else {
                    warn!("Invalid JSON filter: {}", filter.1);
                    true
                }
            }

            "depth" => {
                if let Some(depth) = depth {
                    check_range(&parse_range_input(&filter.1).unwrap(), depth) ^ negated
                } else {
                    warn!("You provided a depth filter but you are not scanning recursively");
                    true
                }
            }

            _ => {
                warn!("Unknown filter: {}", filter.0);
                // We return true so that the filter is not applied
                true
            }
        };

        outs.push(out);
    }

    if opts.or {
        outs.iter().any(|&x| x)
    } else {
        outs.iter().all(|&x| x)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Addition {
    pub key: String,
    pub value: String,
}

pub fn parse_show(opts: &Opts, text: &str, response: &reqwest::Response) -> Vec<Addition> {
    let mut additions: Vec<Addition> = vec![];

    for show in &opts.show {
        match show.as_str() {
            "type" => {
                let is_dir = is_directory(response);
                additions.push(Addition {
                    key: "type".to_string(),
                    value: if is_dir {
                        "directory".to_string()
                    } else {
                        let content_type = response.headers().get(reqwest::header::CONTENT_TYPE);
                        if let Some(content_type) = content_type {
                            content_type.to_str().unwrap().to_string()
                        } else {
                            "unknown".to_string()
                        }
                    },
                });
            }
            "length" | "size" => {
                additions.push(Addition {
                    key: "length".to_string(),
                    value: text.len().to_string(),
                });
            }
            "hash" | "md5" => {
                additions.push(Addition {
                    key: "hash".to_string(),
                    value: format!("{:x}", md5::compute(text)),
                });
            }
            "headers_length" | "headers_size" => {
                additions.push(Addition {
                    key: "headers_length".to_string(),
                    value: response.headers().len().to_string(),
                });
            }
            "headers_hash" => {
                additions.push(Addition {
                    key: "headers_hash".to_string(),
                    value: format!(
                        "{:x}",
                        md5::compute(&response.headers().iter().fold(
                            String::new(),
                            |acc, (key, value)| {
                                format!("{}{}: {}\n", acc, key, value.to_str().unwrap())
                            }
                        ))
                    ),
                });
            }
            "body" | "text" | "content" => {
                additions.push(Addition {
                    key: "body".to_string(),
                    value: text.to_string(),
                });
            }
            "headers" => {
                additions.push(Addition {
                    key: "headers".to_string(),
                    value: response
                        .headers()
                        .iter()
                        .fold("\n".to_string(), |acc, (key, value)| {
                            format!("{}{}: {}\n", acc, key, value.to_str().unwrap())
                        }),
                });
            }
            "cookie" | "cookies" => {
                let headers = response.headers();
                let cookies = headers.get_all(reqwest::header::SET_COOKIE);

                additions.push(Addition {
                    key: "cookies".to_string(),
                    value: cookies.iter().fold("\n".to_string(), |acc, value| {
                        format!("{}{}\n", acc, value.to_str().unwrap_or("Not displayable"))
                    }),
                });
            }
            _ => {}
        }
    }

    additions
}

pub fn print_error(opts: &Opts, progress: &indicatif::ProgressBar, url: &str, err: reqwest::Error) {
    if !opts.quiet {
        if err.is_timeout() {
            progress.println(format!(
                "{} {} {}",
                ERROR.to_string().red(),
                "Timeout reached".bold(),
                url
            ));
        } else if err.is_redirect() {
            progress.println(format!(
                "{} {} {} {}",
                WARNING.to_string().yellow(),
                "Redirect limit reached".bold(),
                url,
                "Check --follow-redirects".dimmed()
            ));
        } else if err.is_connect() {
            progress.println(format!(
                "{} {} {} {}",
                ERROR.to_string().red(),
                "Connection error".bold(),
                url,
                format!("({})", err).dimmed()
            ));
        } else if err.is_request() {
            progress.println(format!(
                "{} {} {} {}",
                ERROR.to_string().red(),
                "Request error".bold(),
                url,
                format!("({})", err).dimmed()
            ));
        } else {
            progress.println(format!(
                "{} {} {} {}",
                ERROR.to_string().red(),
                "Unknown Error".bold(),
                url,
                format!("({})", err).dimmed()
            ));
        }
    }
}

pub fn is_directory(response: &reqwest::Response) -> bool {
    if response.status().is_redirection() {
        // status code is 3xx
        match response.headers().get("Location") {
            // and has a Location header
            Some(loc) => {
                // get absolute redirect Url based on the already known base url
                log::debug!("Location header: {:?}", loc);

                if let Ok(loc_str) = loc.to_str() {
                    if let Ok(abs_url) = response.url().join(loc_str) {
                        if format!("{}/", response.url()) == abs_url.as_str() {
                            // if current response's Url + / == the absolute redirection
                            // location, we've found a directory suitable for recursion
                            log::debug!(
                                "found directory suitable for recursion: {}",
                                response.url()
                            );
                            return true;
                        }
                    }
                }
            }
            None => {
                log::debug!(
                    "expected Location header, but none was found: {:?}",
                    response
                );
                return false;
            }
        }
    } else if response.status().is_success() || matches!(response.status(), StatusCode::FORBIDDEN) {
        // status code is 2xx or 403, need to check if it ends in /

        if response.url().as_str().ends_with('/') {
            log::debug!("{} is directory suitable for recursion", response.url());
            return true;
        }
    }

    false
}
