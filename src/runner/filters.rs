use log::warn;
use serde::{Deserialize, Serialize};

use crate::{
    cli::opts::Opts,
    utils::{check_range, constants::STATUS_CODES, parse_range_input},
};

// Returns true if the response should be kept
pub fn check(
    opts: &Opts,
    res_text: &str,
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
        filters.push(("status".to_string(), STATUS_CODES.to_string()));
        filters
    };

    for filter in filters {
        let mut filter = filter;
        let mut filter_depth: Option<usize> = None;

        // if the filter starts with [depth] then we parse the depth and remove it from the filter
        if filter.0.starts_with("[") {
            let start_index = filter.0.find('[').unwrap();
            let end_index = filter.0.find(']').unwrap();
            let depth = filter.0[start_index + 1..end_index].parse::<usize>();
            filter.0 = filter.0[end_index + 1..].to_string();
            if depth.is_ok() {
                filter_depth = Some(depth.unwrap());
            } else {
                warn!("Invalid depth filter: {}", depth.unwrap_err());
            }
        }

        // If this filter is not for the current depth, we skip it
        if filter_depth.is_some() && !depth.is_some() {
            warn!("You provided a depth filter but you are not scanning recursively");
        }
        if filter_depth.is_some() && depth.is_some() && filter_depth != depth {
            continue;
        }
        let negated = filter.0.starts_with("!");
        let out = match filter.0.trim_start_matches("!") {
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
            _ => true,
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
            "length" | "size" => {
                additions.push(Addition {
                    key: "length".to_string(),
                    value: text.len().to_string(),
                });
            }
            "hash" => {
                additions.push(Addition {
                    key: "hash".to_string(),
                    value: format!("{:x}", md5::compute(&text)),
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
            "body" | "text" => {
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
            _ => {}
        }
    }

    additions
}
