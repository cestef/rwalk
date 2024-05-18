use colored::Colorize;
use log::warn;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    cli::opts::Opts,
    utils::{
        check_range,
        constants::{ERROR, WARNING},
        parse_range_input,
    },
};
use color_eyre::eyre::Result;

// Returns true if the response should be kept
pub fn check(
    opts: &Opts,
    progress: &indicatif::ProgressBar,
    res_text: &str,
    time: u128,
    depth: Option<usize>,
    response: &reqwest::Response,
) -> bool {
    let mut outs: Vec<bool> = Vec::new();

    for filter in opts.filter.clone().iter_mut() {
        // if the filter starts with [depth] then we parse the depth and remove it from the filter
        let filter_depth = if filter.0.starts_with('[') {
            let start_index = filter.0.find('[').unwrap();
            let end_index = filter.0.find(']').unwrap();
            let depth = filter.0[start_index + 1..end_index].parse::<usize>();
            filter.0 = filter.0[end_index + 1..].to_string();
            if let Ok(d) = depth {
                Some(d)
            } else {
                // warn!("Invalid depth filter: {}", depth.unwrap_err());
                progress.println(format!(
                    "{} {} {}",
                    ERROR.to_string().red(),
                    "Invalid depth filter".bold(),
                    depth.unwrap_err()
                ));
                None
            }
        } else {
            None
        };

        // If this filter is not for the current depth, we skip it
        if filter_depth.is_some() && depth.is_none() {
            // warn!("You provided a depth filter but you are not scanning recursively");
            progress.println(format!(
                "{} {}",
                WARNING.to_string().yellow(),
                "You provided a depth filter but you are not scanning recursively".bold()
            ));
        }
        if filter_depth.is_some() && depth.is_some() && filter_depth != depth {
            continue;
        }
        let negated = filter.0.starts_with('!');
        let out = match filter.0.trim_start_matches('!') {
            "time" => check_range(&parse_range_input(&filter.1).unwrap(), time as usize) ^ negated,
            "status" => {
                let status_code = response.status().as_u16();
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
                let headers = response.headers();
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
                            // warn!("Response is not valid JSON: {}", e);
                            progress.println(format!(
                                "{} {} {}",
                                ERROR.to_string().red(),
                                "Response is not valid JSON".bold(),
                                e
                            ));
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
                    // warn!("Invalid JSON filter: {}", filter.1);
                    progress.println(format!(
                        "{} {}",
                        ERROR.to_string().red(),
                        "Invalid JSON filter".bold()
                    ));
                    true
                }
            }

            "depth" => {
                if let Some(depth) = depth {
                    check_range(&parse_range_input(&filter.1).unwrap(), depth) ^ negated
                } else {
                    // warn!("You provided a depth filter but you are not scanning recursively");
                    progress.println(format!(
                        "{} {}",
                        WARNING.to_string().yellow(),
                        "You provided a depth filter but you are not scanning recursively".bold()
                    ));
                    true
                }
            }
            "type" => {
                let is_dir = is_directory(response, res_text);
                if filter.1 == "directory" {
                    is_dir ^ negated
                } else {
                    let headers = response.headers();
                    let content_type = headers.get(reqwest::header::CONTENT_TYPE);
                    if let Some(content_type) = content_type {
                        (content_type.to_str().unwrap() == filter.1) ^ negated
                    } else {
                        false ^ negated
                    }
                }
            }
            "lines" => {
                let lines = res_text.lines().count();
                check_range(&parse_range_input(&filter.1).unwrap(), lines) ^ negated
            }
            // similar:value:threshold 0-100
            "similar" | "similarity" => {
                let split_index = filter.1.find(':');
                if let Some(split_index) = split_index {
                    let (value, threshold) = filter.1.split_at(split_index);
                    let threshold = threshold.trim_start_matches(':');
                    let threshold_range = parse_range_input(threshold);
                    if let Ok(range) = threshold_range {
                        let value = value.trim_end_matches(':');
                        let similarity = strsim::jaro_winkler(value, res_text);
                        check_range(&range, (similarity * 100.0) as usize) ^ negated
                    } else {
                        // warn!("Invalid threshold in filter: {}", filter.1);
                        progress.println(format!(
                            "{} {} {}",
                            ERROR.to_string().red(),
                            "Invalid threshold in filter".bold(),
                            filter.1
                        ));
                        true
                    }
                } else {
                    // warn!("Invalid filter: {}", filter.1);
                    progress.println(format!(
                        "{} {}",
                        ERROR.to_string().red(),
                        "Invalid filter".bold()
                    ));
                    true
                }
            }
            "url" => {
                let url = response.url().as_str();
                if filter.1.starts_with("http://") || filter.1.starts_with("https://") {
                    url.contains(&filter.1) ^ negated
                } else {
                    url.contains(&format!("http://{}", filter.1))
                        || url.contains(&format!("https://{}", filter.1)) ^ negated
                }
            }
            _ => {
                // warn!("Unknown filter: {}", filter.0);
                progress.println(format!(
                    "{} {}",
                    WARNING.to_string().yellow(),
                    "Unknown filter".bold()
                ));
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
        // Check if the show filter is a key:value pair
        let show = if let Some(split_index) = show.find(':') {
            let (key, value) = show.split_at(split_index);
            let value = value.trim_start_matches(':');
            (key, value)
        } else {
            (show.as_str(), "")
        };

        match show.0.to_lowercase().as_str() {
            "type" => {
                let is_dir = is_directory(response, text);
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
            "similar" | "similarity" => {
                // similar:value
                let similarity = strsim::jaro_winkler(show.1, text);
                additions.push(Addition {
                    key: "similarity".to_string(),
                    value: format!("{}%", (similarity * 100.0) as usize),
                });
            }
            _ => {}
        }
    }

    additions
}

pub fn print_error(
    opts: &Opts,
    print_fn: impl FnOnce(String) -> Result<()>,
    url: &str,
    err: reqwest::Error,
) -> Result<()> {
    if !opts.quiet {
        if err.is_timeout() {
            print_fn(format!(
                "{} {} {}",
                ERROR.to_string().red(),
                "Timeout reached".bold(),
                url
            ))?;
        } else if err.is_redirect() {
            print_fn(format!(
                "{} {} {} {}",
                WARNING.to_string().yellow(),
                "Redirect limit reached".bold(),
                url,
                "Check --follow-redirects".dimmed()
            ))?;
        } else if err.is_connect() {
            print_fn(format!(
                "{} {} {} {}",
                ERROR.to_string().red(),
                "Connection error".bold(),
                url,
                format!("({})", err).dimmed()
            ))?;
        } else if err.is_request() {
            print_fn(format!(
                "{} {} {} {}",
                ERROR.to_string().red(),
                "Request error".bold(),
                url,
                format!("({})", err).dimmed()
            ))?;
        } else {
            print_fn(format!(
                "{} {} {} {}",
                ERROR.to_string().red(),
                "Unknown Error".bold(),
                url,
                format!("({})", err).dimmed()
            ))?;
        }
    }
    Ok(())
}

pub fn is_html_directory(body: &str) -> bool {
    let body = body.to_lowercase();
    // Apache
    if body.contains("index of") {
        return true;
    }
    // Nginx
    if body.contains("name=\"description\" content=\"nginx directory listing\"") {
        return true;
    }
    // ASP.NET
    if body.contains("directory listing -- /") {
        return true;
    }
    // Tomcat
    if body.contains("directory listing for /") {
        return true;
    }

    false
}

pub fn is_directory(response: &reqwest::Response, body: &str) -> bool {
    if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
        if content_type.to_str().unwrap().starts_with("text/html") {
            // log::debug!("{} is HTML", response.url());
            if is_html_directory(body) {
                log::debug!("{} is directory suitable for recursion", response.url());
                return true;
            }
        }
    }
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
    } else if response.status().is_success()
        || matches!(
            response.status(),
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED // 403, 401 ; a little bit of a hack but it works most of the time
        )
    {
        // status code is 2xx or 403, need to check if it ends in /

        if response.url().as_str().ends_with('/') {
            log::debug!("{} is directory suitable for recursion", response.url());
            return true;
        }
    }

    false
}
