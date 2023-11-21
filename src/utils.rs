use anyhow::Result;
use colored::Colorize;
use std::io::{Read, Write};

use crate::{cli::OPTS, constants::BANNER_STR};

pub fn parse_wordlists(wordlists: &Vec<String>) -> Vec<String> {
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
    if OPTS.filter_contains.is_some() {
        let filter_contains = OPTS.filter_contains.clone().unwrap();
        words.retain(|word| word.contains(&filter_contains));
    }
    if OPTS.filter_starts_with.is_some() {
        let filter_starts_with = OPTS.filter_starts_with.clone().unwrap();
        words.retain(|word| word.starts_with(&filter_starts_with));
    }
    if OPTS.filter_ends_with.is_some() {
        let filter_ends_with = OPTS.filter_ends_with.clone().unwrap();
        words.retain(|word| word.ends_with(&filter_ends_with));
    }
    if OPTS.filter_regex.is_some() {
        let filter_regex = OPTS.filter_regex.clone().unwrap();
        let re = regex::Regex::new(&filter_regex)?;
        words.retain(|word| re.is_match(&word));
    }
    if OPTS.filter_length.is_some() {
        let filter_length = OPTS.filter_length.clone().unwrap();
        words.retain(|word| word.len() == filter_length);
    }
    if OPTS.filter_min_length.is_some() {
        let filter_min_length = OPTS.filter_min_length.clone().unwrap();
        words.retain(|word| word.len() >= filter_min_length);
    }
    if OPTS.filter_max_length.is_some() {
        let filter_max_length = OPTS.filter_max_length.clone().unwrap();
        words.retain(|word| word.len() <= filter_max_length);
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
