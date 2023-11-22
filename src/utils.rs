use anyhow::Result;
use colored::Colorize;
use std::{
    io::{Read, Write},
    path::PathBuf,
};

use crate::{cli::OPTS, constants::BANNER_STR};

pub fn parse_wordlists(wordlists: &Vec<PathBuf>) -> Vec<String> {
    let mut wordlist = Vec::new();
    for wordlist_path in wordlists {
        let mut file = std::fs::File::open(wordlist_path).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();

        let contents = unsafe { String::from_utf8_unchecked(bytes) };

        for word in contents.split("\n") {
            let word = word.trim();
            if word.len() > 0 {
                wordlist.push(word.to_string());
            }
        }
    }
    wordlist
}

pub fn banner() {
    println!("{}", BANNER_STR.to_string().bold().bright_red());
    println!(
        "{} {}",
        "Version:".dimmed(),
        env!("CARGO_PKG_VERSION").dimmed().bold()
    );
    println!("{} {}", "Author:".dimmed(), "cstef".dimmed().bold());
    println!("");
}

pub fn hide_cursor() {
    print!("\x1B[?25l");
    std::io::stdout().flush().unwrap();
}

pub fn show_cursor() {
    print!("\x1B[?25h");
    std::io::stdout().flush().unwrap();
}
pub fn get_emoji_for_status_code_colored(status_code: u16) -> String {
    match status_code {
        200..=299 => "✓".green().to_string(),
        300..=399 => "⇝".blue().to_string(),
        400..=403 => "✖".red().to_string(),
        500..=599 => "⚠".yellow().to_string(),
        _ => "⚠".yellow().to_string(),
    }
}

pub fn get_emoji_for_status_code(status_code: u16) -> String {
    match status_code {
        200..=299 => "✓".to_string(),
        300..=399 => "⇝".to_string(),
        400..=403 => "✖".to_string(),
        500..=599 => "⚠".to_string(),
        _ => "⚠".to_string(),
    }
}

pub fn apply_filters(words: &mut Vec<String>) -> Result<()> {
    if OPTS.wordlist_filter_contains.is_some() {
        let filter_contains = OPTS.wordlist_filter_contains.clone().unwrap();
        words.retain(|word| word.contains(&filter_contains));
    }
    if OPTS.wordlist_filter_starts_with.is_some() {
        let filter_starts_with = OPTS.wordlist_filter_starts_with.clone().unwrap();
        words.retain(|word| word.starts_with(&filter_starts_with));
    }
    if OPTS.wordlist_filter_ends_with.is_some() {
        let filter_ends_with = OPTS.wordlist_filter_ends_with.clone().unwrap();
        words.retain(|word| word.ends_with(&filter_ends_with));
    }
    if OPTS.wordlist_filter_regex.is_some() {
        let filter_regex = OPTS.wordlist_filter_regex.clone().unwrap();
        let re = regex::Regex::new(&filter_regex)?;
        words.retain(|word| re.is_match(&word));
    }
    if OPTS.wordlist_filter_length.is_some() {
        let filter_length = OPTS.wordlist_filter_length.clone().unwrap();
        let parsed_filter_length = parse_range_input(&filter_length).unwrap();
        words.retain(|word| check_range(&parsed_filter_length, word.len()));
    }

    Ok(())
}

pub fn apply_transformations(words: &mut Vec<String>) {
    if OPTS.transform_lower {
        words.iter_mut().for_each(|word| {
            *word = word.to_lowercase();
        });
    }
    if OPTS.transform_upper {
        words.iter_mut().for_each(|word| {
            *word = word.to_uppercase();
        });
    }
    if OPTS.transform_prefix.is_some() {
        let transform_prefix = OPTS.transform_prefix.clone().unwrap();
        words.iter_mut().for_each(|word| {
            *word = format!("{}{}", transform_prefix, word);
        });
    }
    if OPTS.transform_suffix.is_some() {
        let transform_suffix = OPTS.transform_suffix.clone().unwrap();
        words.iter_mut().for_each(|word| {
            *word = format!("{}{}", word, transform_suffix);
        });
    }
    if OPTS.transform_capitalize {
        words.iter_mut().for_each(|word| {
            *word = word.to_lowercase();
            let mut chars = word.chars();
            if let Some(first_char) = chars.next() {
                *word = format!("{}{}", first_char.to_uppercase(), chars.as_str());
            }
        });
    }
}

pub fn is_response_filtered(res_text: &str, status_code: u16, time: u16) -> bool {
    if OPTS.filter_time.is_some() {
        let filter_time = OPTS.filter_time.clone().unwrap();
        let parsed_filter_time = parse_range_input(&filter_time).unwrap();
        if !check_range(&parsed_filter_time, time as usize) {
            return false;
        }
    }
    if OPTS.filter_status_code.is_some() {
        let filter_status_code = OPTS.filter_status_code.clone().unwrap();
        let parsed_filter_status_code = parse_range_input(&filter_status_code).unwrap();
        if !check_range(&parsed_filter_status_code, status_code as usize) {
            return false;
        }
    }
    if OPTS.filter_contains.is_some() {
        let filter_contains = OPTS.filter_contains.clone().unwrap();
        if !res_text.contains(&filter_contains) {
            return false;
        }
    }
    if OPTS.filter_starts_with.is_some() {
        let filter_starts_with = OPTS.filter_starts_with.clone().unwrap();
        if !res_text.starts_with(&filter_starts_with) {
            return false;
        }
    }
    if OPTS.filter_ends_with.is_some() {
        let filter_ends_with = OPTS.filter_ends_with.clone().unwrap();
        if !res_text.ends_with(&filter_ends_with) {
            return false;
        }
    }
    if OPTS.filter_regex.is_some() {
        let filter_regex = OPTS.filter_regex.clone().unwrap();
        let re = regex::Regex::new(&filter_regex).unwrap();
        if !re.is_match(&res_text) {
            return false;
        }
    }
    if OPTS.filter_length.is_some() {
        let filter_length = OPTS.filter_length.clone().unwrap();
        let parsed_filter_length = parse_range_input(&filter_length).unwrap();
        if !check_range(&parsed_filter_length, res_text.len()) {
            return false;
        }
    }

    true
}
pub fn check_range(ranges: &Vec<(usize, usize)>, num: usize) -> bool {
    for range in ranges {
        if num >= range.0 && num <= range.1 {
            return true;
        }
    }
    false
}
pub fn parse_range_input(s: &str) -> Result<Vec<(usize, usize)>, String> {
    let mut ranges = Vec::new();
    let parts = s.split(",").collect::<Vec<_>>();
    for part in parts {
        if part.is_empty() {
            continue;
        }
        if part.starts_with(">") {
            let num = part[1..].parse::<usize>();
            match num {
                Ok(num) => ranges.push((num, usize::MAX)),
                Err(_) => return Err("Invalid range".to_string()),
            }
        } else if part.starts_with("<") {
            let num = part[1..].parse::<usize>();
            match num {
                Ok(num) => ranges.push((0, num)),
                Err(_) => return Err("Invalid range".to_string()),
            }
        } else {
            let part = part.trim();
            let parts = part.split("-").collect::<Vec<_>>();
            if parts.len() == 1 {
                let num = parts[0].parse::<usize>();
                match num {
                    Ok(num) => ranges.push((num, num)),
                    Err(_) => return Err("Invalid range".to_string()),
                }
            } else if parts.len() == 2 {
                let num1 = parts[0].parse::<usize>();
                let num2 = parts[1].parse::<usize>();
                match (num1, num2) {
                    (Ok(num1), Ok(num2)) => ranges.push((num1, num2)),
                    _ => return Err("Invalid range".to_string()),
                }
            } else {
                return Err("Invalid range".to_string());
            }
        }
    }
    Ok(ranges)
}
