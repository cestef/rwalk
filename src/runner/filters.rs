use serde::{Deserialize, Serialize};

use crate::{
    cli::opts::Opts,
    utils::{check_range, constants::STATUS_CODES, parse_range_input},
};

// Returns true if the response should be kept
pub fn check(opts: &Opts, res_text: &str, status_code: u16, time: u128) -> bool {
    let mut outs: Vec<bool> = Vec::new();

    let filters = if opts.filter.iter().any(|e| e.0 == "status") {
        opts.filter.clone()
    } else {
        let mut filters = opts.filter.clone();
        filters.push(("status".to_string(), STATUS_CODES.to_string()));
        filters
    };

    for filter in filters {
        let not = filter.0.starts_with("!");
        let out = match filter.0.trim_start_matches("!") {
            "time" => {
                let parsed_filter_time = parse_range_input(&filter.1).unwrap();
                if !check_range(&parsed_filter_time, time as usize) {
                    not
                } else {
                    !not
                }
            }
            "status" => {
                let parsed_filter_status_code = parse_range_input(&filter.1).unwrap();
                if !check_range(&parsed_filter_status_code, status_code as usize) {
                    not
                } else {
                    !not
                }
            }
            "contains" => {
                if !res_text.contains(&filter.1) {
                    not
                } else {
                    !not
                }
            }
            "starts" => {
                if !res_text.starts_with(&filter.1) {
                    not
                } else {
                    !not
                }
            }
            "ends" => {
                if !res_text.ends_with(&filter.1) {
                    not
                } else {
                    !not
                }
            }
            "regex" => {
                let re = regex::Regex::new(&filter.1).unwrap();
                if !re.is_match(res_text) {
                    not
                } else {
                    !not
                }
            }
            "length" => {
                let parsed_filter_length = parse_range_input(&filter.1).unwrap();
                if !check_range(&parsed_filter_length, res_text.len()) {
                    not
                } else {
                    !not
                }
            }
            "hash" => {
                let hash = md5::compute(res_text);
                if !filter.1.contains(&format!("{:x}", hash)) {
                    not
                } else {
                    !not
                }
            }
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
            "length" => {
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
            "headers_length" => {
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
            "body" => {
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
