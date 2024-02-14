use anyhow::{Context, Result};
use colored::Colorize;
use tokio::io::AsyncReadExt;

use crate::{
    cli::Opts,
    utils::{check_range, parse_range_input},
};

pub async fn parse(wordlists: &Vec<String>) -> Result<Vec<String>> {
    let mut wordlist = Vec::new();
    for wordlist_path in wordlists {
        let words: String = match wordlist_path.as_str() {
            "-" => {
                let mut stdin = tokio::io::stdin();

                let mut buf = String::new();
                stdin.read_to_string(&mut buf).await?;
                buf
            }
            _ => {
                let mut file = tokio::fs::File::open(wordlist_path)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to open wordlist file: {}",
                            wordlist_path.to_string().bold().red()
                        )
                    })?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes).await?;

                unsafe { String::from_utf8_unchecked(bytes) }
            }
        };
        wordlist.extend(
            words
                .split('\n')
                .map(|x| x.to_string())
                .filter(|x| !x.is_empty())
                .collect::<Vec<String>>(),
        );
    }
    Ok(wordlist)
}

pub fn filters(opts: &Opts, words: &mut Vec<String>) -> Result<()> {
    for filter in &opts.wordlist_filter {
        let not = filter.0.starts_with("!");
        match filter.0.trim_start_matches("!") {
            "contains" => {
                words.retain(|word| {
                    if not {
                        !word.contains(&filter.1)
                    } else {
                        word.contains(&filter.1)
                    }
                });
            }
            "starts" => {
                words.retain(|word| {
                    if not {
                        !word.starts_with(&filter.1)
                    } else {
                        word.starts_with(&filter.1)
                    }
                });
            }
            "ends" => {
                words.retain(|word| {
                    if not {
                        !word.ends_with(&filter.1)
                    } else {
                        word.ends_with(&filter.1)
                    }
                });
            }
            "regex" => {
                let re = regex::Regex::new(&filter.1)?;
                words.retain(|word| {
                    if not {
                        !re.is_match(word)
                    } else {
                        re.is_match(word)
                    }
                });
            }
            "length" => {
                let parsed_filter_length = parse_range_input(&filter.1)?;
                words.retain(|word| {
                    if not {
                        !check_range(&parsed_filter_length, word.len())
                    } else {
                        check_range(&parsed_filter_length, word.len())
                    }
                });
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn transformations(opts: &Opts, words: &mut Vec<String>) {
    for transformation in &opts.transform {
        match transformation.0.as_str() {
            "lower" => {
                words.iter_mut().for_each(|word| {
                    *word = word.to_lowercase();
                });
            }
            "upper" => {
                words.iter_mut().for_each(|word| {
                    *word = word.to_uppercase();
                });
            }
            "prefix" => {
                let transform_prefix = transformation.1.clone().unwrap();
                words.iter_mut().for_each(|word| {
                    *word = format!("{}{}", transform_prefix, word);
                });
            }
            "suffix" => {
                let transform_suffix = transformation.1.clone().unwrap();
                words.iter_mut().for_each(|word| {
                    *word = format!("{}{}", word, transform_suffix);
                });
            }
            "capitalize" => {
                words.iter_mut().for_each(|word| {
                    *word = word.to_lowercase();
                    let mut chars = word.chars();
                    if let Some(first_char) = chars.next() {
                        *word = format!("{}{}", first_char.to_uppercase(), chars.as_str());
                    }
                });
            }
            "reverse" => {
                words.iter_mut().for_each(|word| {
                    *word = word.chars().rev().collect::<String>();
                });
            }
            "remove" => {
                let transform_remove = transformation.1.clone().unwrap();
                words.iter_mut().for_each(|word| {
                    *word = word.replace(&transform_remove, "");
                });
            }
            "replace" => {
                let transform_replace = transformation.1.clone().unwrap();
                let parts = transform_replace.split("=").collect::<Vec<_>>();
                if parts.len() == 2 {
                    words.iter_mut().for_each(|word| {
                        *word = word.replace(parts[0], parts[1]);
                    });
                }
            }
            _ => {}
        }
    }
}
