use std::collections::HashMap;

use anyhow::{Context, Result};
use colored::Colorize;
use tokio::io::AsyncReadExt;

use crate::{
    cli::opts::Opts,
    utils::{check_range, constants::DEFAULT_FUZZ_KEY, parse_range_input},
};

pub async fn parse(wordlists: &Vec<(String, Vec<String>)>) -> Result<HashMap<String, Vec<String>>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for (path, keys) in wordlists {
        let words: String = match path.as_str() {
            "-" => {
                let mut stdin = tokio::io::stdin();

                let mut buf = String::new();
                stdin.read_to_string(&mut buf).await?;
                buf
            }
            _ => {
                let mut file = tokio::fs::File::open(path.clone()).await.with_context(|| {
                    format!(
                        "Failed to open wordlist file: {}",
                        path.to_string().bold().red()
                    )
                })?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes).await?;

                unsafe { String::from_utf8_unchecked(bytes) }
            }
        };
        for key in {
            if keys.is_empty() {
                vec![DEFAULT_FUZZ_KEY.to_string()]
            } else {
                keys.clone()
            }
        } {
            out.entry(key.clone()).or_default().extend(
                words
                    .split('\n')
                    .map(|x| x.to_string())
                    .filter(|x| !x.is_empty()),
            );
        }
    }

    Ok(out)
}

pub fn deduplicate(wordlists: &mut HashMap<String, Vec<String>>) {
    for words in (*wordlists).values_mut() {
        words.sort();
        words.dedup();
    }
}

pub fn filters(opts: &Opts, wordlists: &mut HashMap<String, Vec<String>>) -> Result<()> {
    for filter in &opts.wordlist_filter {
        let not = filter.0.starts_with('!');
        match filter.0.trim_start_matches('!') {
            "contains" => {
                for words in (*wordlists).values_mut() {
                    words.retain(|word| {
                        if not {
                            !word.contains(&filter.1)
                        } else {
                            word.contains(&filter.1)
                        }
                    });
                }
            }
            "starts" => {
                for words in (*wordlists).values_mut() {
                    words.retain(|word| {
                        if not {
                            !word.starts_with(&filter.1)
                        } else {
                            word.starts_with(&filter.1)
                        }
                    });
                }
            }
            "ends" => {
                for words in (*wordlists).values_mut() {
                    words.retain(|word| {
                        if not {
                            !word.ends_with(&filter.1)
                        } else {
                            word.ends_with(&filter.1)
                        }
                    });
                }
            }
            "regex" => {
                let re = regex::Regex::new(&filter.1)?;
                for words in (*wordlists).values_mut() {
                    words.retain(|word| {
                        if not {
                            !re.is_match(word)
                        } else {
                            re.is_match(word)
                        }
                    });
                }
            }
            "length" => {
                let parsed_filter_length = parse_range_input(&filter.1)?;
                for words in (*wordlists).values_mut() {
                    words.retain(|word| {
                        if not {
                            !check_range(&parsed_filter_length, word.len())
                        } else {
                            check_range(&parsed_filter_length, word.len())
                        }
                    });
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn transformations(opts: &Opts, wordlists: &mut HashMap<String, Vec<String>>) {
    for transformation in &opts.transform {
        match transformation.0.as_str() {
            "lower" => {
                for words in (*wordlists).values_mut() {
                    words.iter_mut().for_each(|word| {
                        *word = word.to_lowercase();
                    });
                }
            }
            "upper" => {
                for words in (*wordlists).values_mut() {
                    words.iter_mut().for_each(|word| {
                        *word = word.to_uppercase();
                    });
                }
            }
            "prefix" => {
                let transform_prefix = transformation.1.clone().unwrap();
                for words in (*wordlists).values_mut() {
                    words.iter_mut().for_each(|word| {
                        *word = format!("{}{}", transform_prefix, word);
                    });
                }
            }
            "suffix" => {
                let transform_suffix = transformation.1.clone().unwrap();
                for words in (*wordlists).values_mut() {
                    words.iter_mut().for_each(|word| {
                        *word = format!("{}{}", word, transform_suffix);
                    });
                }
            }
            "capitalize" => {
                for words in (*wordlists).values_mut() {
                    words.iter_mut().for_each(|word| {
                        *word = word.to_lowercase();
                        let mut chars = word.chars();
                        if let Some(first_char) = chars.next() {
                            *word = format!("{}{}", first_char.to_uppercase(), chars.as_str());
                        }
                    });
                }
            }
            "reverse" => {
                for words in (*wordlists).values_mut() {
                    words.iter_mut().for_each(|word| {
                        *word = word.chars().rev().collect::<String>();
                    });
                }
            }
            "remove" => {
                let transform_remove = transformation.1.clone().unwrap();
                for words in (*wordlists).values_mut() {
                    words.iter_mut().for_each(|word| {
                        *word = word.replace(&transform_remove, "");
                    });
                }
            }
            "replace" => {
                let transform_replace = transformation.1.clone().unwrap();
                let parts = transform_replace.split('=').collect::<Vec<_>>();
                if parts.len() == 2 {
                    for words in (*wordlists).values_mut() {
                        words.iter_mut().for_each(|word| {
                            *word = word.replace(parts[0], parts[1]);
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn compute_checksum(wordlists: &HashMap<String, Vec<String>>) -> String {
    format!(
        "{:x}",
        md5::compute(
            wordlists
                .iter()
                .map(|(key, words)| format!("{}:{:?}", key, words.join(",")))
                .collect::<Vec<String>>()
                .join("|")
        )
    )
}
