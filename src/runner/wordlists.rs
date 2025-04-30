use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Context, Result};
use colored::Colorize;
use tokio::io::AsyncReadExt;

use crate::{
    cli::opts::{Opts, Wordlist},
    utils::{check_range, constants::DEFAULT_FUZZ_KEY, parse_range_input},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParsedWordlist {
    pub path: String,
    pub words: Vec<String>,
}

impl ParsedWordlist {
    pub fn new(path: String, words: Vec<String>) -> Self {
        Self { path, words }
    }
}

/// Parse wordlists
///
/// # Arguments
///
/// * `wordlists` - The paths to wordlists to parse
/// * `include_comments` - Whether to include comment lines (starting with '#')
///
/// # Returns
///
/// A hashmap of parsed wordlists (key = path, value = ParsedWordlist)
/// Where ParsedWordlist contains the path to the wordlist and the words in the wordlist
pub async fn parse(
    wordlists: &[Wordlist],
    include_comments: bool,
) -> Result<HashMap<String, ParsedWordlist>> {
    let mut out: HashMap<String, ParsedWordlist> = HashMap::new();

    for Wordlist(path, keys) in wordlists {
        let bytes = match path.as_str() {
            "-" => {
                let mut stdin = tokio::io::stdin();
                let mut buffer = Vec::new();
                stdin.read_to_end(&mut buffer).await?;
                buffer
            }
            _ => {
                let expanded_path = expand_tilde(Path::new(path))?;
                let canonical_path = expanded_path.canonicalize().with_context(|| {
                    format!("Failed to canonicalize path: {}", path.bold().red())
                })?;

                tokio::fs::read(&canonical_path).await.with_context(|| {
                    format!("Failed to open wordlist file: {}", path.bold().red())
                })?
            }
        };

        let content = String::from_utf8_lossy(&bytes);

        let keys_to_use = if keys.is_empty() {
            vec![DEFAULT_FUZZ_KEY.to_string()]
        } else {
            keys.iter().map(ToString::to_string).collect()
        };

        // filter lines before processing them for each key to avoid redundant filtering
        let filtered_lines: Vec<String> = content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                if include_comments {
                    Some(line.to_string())
                } else {
                    strip_comments(line)
                }
            })
            .collect();

        for key in keys_to_use {
            out.entry(key)
                .or_insert(ParsedWordlist {
                    path: path.clone(),
                    words: Vec::new(),
                })
                .words
                .extend(filtered_lines.iter().cloned());
        }
    }

    Ok(out)
}

pub fn deduplicate(wordlists: &mut HashMap<String, ParsedWordlist>) {
    for ParsedWordlist { words, .. } in (*wordlists).values_mut() {
        words.sort_unstable();
        words.dedup();
    }
}

pub fn filters(opts: &Opts, wordlists: &mut HashMap<String, ParsedWordlist>) -> Result<()> {
    for filter in opts.wordlist_filter.iter().cloned() {
        let mut filter = filter;
        // if the filter starts with [specifier] then we parse the specifier and remove it from the filter
        let wordlist_specifier = if filter.0.starts_with('[') {
            let start_index = filter.0.find('[').unwrap();
            let end_index = filter.0.find(']').unwrap();
            let specifier = filter.0[start_index + 1..end_index].to_string();
            filter.0 = filter.0[end_index + 1..].to_string();
            Some(specifier)
        } else {
            None
        };
        let not = filter.0.starts_with('!');
        match filter.0.trim_start_matches('!') {
            "contains" => {
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
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
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
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
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
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
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
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
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
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

pub fn transformations(opts: &Opts, wordlists: &mut HashMap<String, ParsedWordlist>) {
    for transformation in opts.transform.clone() {
        let mut transformation = transformation;

        let wordlist_specifier = if transformation.0.starts_with('[') {
            let start_index = transformation.0.find('[').unwrap();
            let end_index = transformation.0.find(']').unwrap();
            let specifier = transformation.0[start_index + 1..end_index].to_string();
            transformation.0 = transformation.0[end_index + 1..].to_string();
            Some(specifier)
        } else {
            None
        };
        match transformation.0.as_str() {
            "lower" => {
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
                    words.iter_mut().for_each(|word| {
                        *word = word.to_lowercase();
                    });
                }
            }
            "upper" => {
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
                    words.iter_mut().for_each(|word| {
                        *word = word.to_uppercase();
                    });
                }
            }
            "prefix" => {
                let transform_prefix = transformation.1.clone().unwrap();
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
                    words.iter_mut().for_each(|word| {
                        *word = format!("{}{}", transform_prefix, word);
                    });
                }
            }
            "suffix" => {
                let transform_suffix = transformation.1.clone().unwrap();
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
                    words.iter_mut().for_each(|word| {
                        *word = format!("{}{}", word, transform_suffix);
                    });
                }
            }
            "capitalize" => {
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
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
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
                    words.iter_mut().for_each(|word| {
                        *word = word.chars().rev().collect::<String>();
                    });
                }
            }
            "remove" => {
                let transform_remove = transformation.1.clone().unwrap();
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
                    words.iter_mut().for_each(|word| {
                        *word = word.replace(&transform_remove, "");
                    });
                }
            }
            "replace" => {
                let transform_replace = transformation.1.clone().unwrap();
                let parts = transform_replace.split('=').collect::<Vec<_>>();
                if parts.len() == 2 {
                    for ParsedWordlist { words, .. } in
                        wordlists.iter_mut().filter_map(|(key, value)| {
                            if let Some(ref specifier) = wordlist_specifier {
                                if specifier == key {
                                    Some(value)
                                } else {
                                    None
                                }
                            } else {
                                Some(value)
                            }
                        })
                    {
                        words.iter_mut().for_each(|word| {
                            *word = word.replace(parts[0], parts[1]);
                        });
                    }
                }
            }
            "encode" => {
                for ParsedWordlist { words, .. } in
                    wordlists.iter_mut().filter_map(|(key, value)| {
                        if let Some(ref specifier) = wordlist_specifier {
                            if specifier == key {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            Some(value)
                        }
                    })
                {
                    words.iter_mut().for_each(|word| {
                        *word = urlencoding::encode(word).to_string();
                    });
                }
            }
            _ => {}
        }
    }
}

pub fn compute_checksum(wordlists: &HashMap<String, ParsedWordlist>) -> String {
    let to_compute = wordlists
        .iter()
        .map(|(key, ParsedWordlist { words, .. })| format!("{}:{:?}", key, words.join(",")))
        .collect::<Vec<String>>()
        .join("|");
    format!("{:x}", md5::compute(to_compute))
}

fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Result<PathBuf> {
    let p = path_user_input.as_ref();
    if !p.starts_with("~") {
        return Ok(p.to_path_buf());
    }
    if p == Path::new("~") {
        return dirs::home_dir().ok_or_else(|| {
            color_eyre::eyre::eyre!("Failed to expand tilde in path: {}", p.display())
        });
    }
    dirs::home_dir()
        .map(|mut h| {
            if h == Path::new("/") {
                // Corner case: `h` root directory;
                // don't prepend extra `/`, just drop the tilde.
                p.strip_prefix("~").unwrap().to_path_buf()
            } else {
                h.push(p.strip_prefix("~/").unwrap());
                h
            }
        })
        .ok_or_else(|| color_eyre::eyre::eyre!("Failed to expand tilde in path: {}", p.display()))
}

// Ref: https://github.com/ffuf/ffuf/blob/57da720af7d1b66066cbbde685b49948f886b29c/pkg/input/wordlist.go#L173
fn strip_comments(text: &str) -> Option<String> {
    // If the line starts with # (ignoring leading whitespace), return None
    if text.trim_start().starts_with('#') {
        return None;
    }

    // Find the position of "#" after a space
    if let Some(index) = text.find(" #") {
        Some(text[..index].to_string())
    } else {
        Some(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::helpers::{KeyOrKeyVal, KeyVal};

    use super::*;

    #[tokio::test]
    async fn test_parse() {
        let wordlists = vec![
            Wordlist(
                "tests/wordlists/micro1.txt".to_string(),
                vec!["W1".to_string()],
            ),
            Wordlist(
                "tests/wordlists/micro2.txt".to_string(),
                vec!["W1".to_string()],
            ),
            Wordlist(
                "tests/wordlists/micro3.txt".to_string(),
                vec!["W2".to_string()],
            ),
            Wordlist(
                "tests/wordlists/micro4comments.txt".to_string(),
                vec!["W4".to_string()],
            ),
        ];
        let parsed = parse(&wordlists, false).await.unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed.get("W1").unwrap().words.len(), 7);
        assert_eq!(parsed.get("W2").unwrap().words.len(), 2);
        assert_eq!(parsed.get("W4").unwrap().words.len(), 3);
    }

    #[test]
    fn test_deduplicate() {
        let mut wordlists = HashMap::new();
        wordlists.insert(
            "FUZZ".to_string(),
            ParsedWordlist::new("".to_string(), vec!["a".to_string(), "b".to_string()]),
        );
        deduplicate(&mut wordlists);
        assert_eq!(wordlists.get("FUZZ").unwrap().words.len(), 2);
        wordlists.insert(
            "FUZZ".to_string(),
            ParsedWordlist::new("".to_string(), vec!["a".to_string(), "a".to_string()]),
        );
        deduplicate(&mut wordlists);
        assert_eq!(wordlists.get("FUZZ").unwrap().words.len(), 1);
    }

    #[test]
    fn test_filters() {
        let mut wordlists = HashMap::new();
        wordlists.insert(
            "FUZZ".to_string(),
            ParsedWordlist::new(
                "".to_string(),
                vec!["ab".to_string(), "a".to_string(), "b".to_string()],
            ),
        );
        filters(
            &Opts {
                wordlist_filter: vec![KeyVal("contains".to_string(), "a".to_string())],
                ..Default::default()
            },
            &mut wordlists,
        )
        .unwrap();
        assert_eq!(wordlists.get("FUZZ").unwrap().words.len(), 2);

        filters(
            &Opts {
                wordlist_filter: vec![KeyVal("length".to_string(), "1".to_string())],
                ..Default::default()
            },
            &mut wordlists,
        )
        .unwrap();
        assert_eq!(wordlists.get("FUZZ").unwrap().words.len(), 1);
    }

    #[test]
    fn test_transformations() {
        let mut wordlists = HashMap::new();
        wordlists.insert(
            "FUZZ".to_string(),
            ParsedWordlist::new("".to_string(), vec!["a".to_string(), "b".to_string()]),
        );
        transformations(
            &Opts {
                transform: vec![KeyOrKeyVal("upper".to_string(), None)],
                ..Default::default()
            },
            &mut wordlists,
        );
        assert_eq!(wordlists.get("FUZZ").unwrap().words.len(), 2);
        assert_eq!(wordlists.get("FUZZ").unwrap().words[0], "A");

        transformations(
            &Opts {
                transform: vec![KeyOrKeyVal("prefix".to_string(), Some("c".to_string()))],
                ..Default::default()
            },
            &mut wordlists,
        );
        assert_eq!(wordlists.get("FUZZ").unwrap().words.len(), 2);
        assert_eq!(wordlists.get("FUZZ").unwrap().words[0], "cA");

        transformations(
            &Opts {
                transform: vec![KeyOrKeyVal("suffix".to_string(), Some("d".to_string()))],
                ..Default::default()
            },
            &mut wordlists,
        );

        assert_eq!(wordlists.get("FUZZ").unwrap().words.len(), 2);
        assert_eq!(wordlists.get("FUZZ").unwrap().words[0], "cAd");

        transformations(
            &Opts {
                transform: vec![KeyOrKeyVal("capitalize".to_string(), None)],
                ..Default::default()
            },
            &mut wordlists,
        );

        assert_eq!(wordlists.get("FUZZ").unwrap().words.len(), 2);
        assert_eq!(wordlists.get("FUZZ").unwrap().words[0], "Cad");
    }

    #[test]
    fn test_compute_checksum() {
        let mut wordlists = HashMap::new();
        wordlists.insert(
            "FUZZ".to_string(),
            ParsedWordlist::new("".to_string(), vec!["a".to_string(), "b".to_string()]),
        );
        assert_eq!(
            compute_checksum(&wordlists),
            "0da67572922cb261bf70d946f2ba6c03"
        );
    }
}
