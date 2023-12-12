use anyhow::Result;
use colored::Colorize;
use parking_lot::Mutex;
use std::{
    io::{Read, Write},
    sync::Arc,
};

use crate::{
    cli::Opts,
    constants::BANNER_STR,
    tree::{Tree, TreeData, TreeNode},
};

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

pub fn apply_filters(opts: &Opts, words: &mut Vec<String>) -> Result<()> {
    if opts.wordlist_filter_contains.is_some() {
        let filter_contains = opts.wordlist_filter_contains.clone().unwrap();
        words.retain(|word| word.contains(&filter_contains));
    }
    if opts.wordlist_filter_starts_with.is_some() {
        let filter_starts_with = opts.wordlist_filter_starts_with.clone().unwrap();
        words.retain(|word| word.starts_with(&filter_starts_with));
    }
    if opts.wordlist_filter_ends_with.is_some() {
        let filter_ends_with = opts.wordlist_filter_ends_with.clone().unwrap();
        words.retain(|word| word.ends_with(&filter_ends_with));
    }
    if opts.wordlist_filter_regex.is_some() {
        let filter_regex = opts.wordlist_filter_regex.clone().unwrap();
        let re = regex::Regex::new(&filter_regex)?;
        words.retain(|word| re.is_match(&word));
    }
    if opts.wordlist_filter_length.is_some() {
        let filter_length = opts.wordlist_filter_length.clone().unwrap();
        let parsed_filter_length = parse_range_input(&filter_length).unwrap();
        words.retain(|word| check_range(&parsed_filter_length, word.len()));
    }

    Ok(())
}

pub fn apply_transformations(opts: &Opts, words: &mut Vec<String>) {
    if opts.transform_lower {
        words.iter_mut().for_each(|word| {
            *word = word.to_lowercase();
        });
    }
    if opts.transform_upper {
        words.iter_mut().for_each(|word| {
            *word = word.to_uppercase();
        });
    }
    if opts.transform_prefix.is_some() {
        let transform_prefix = opts.transform_prefix.clone().unwrap();
        words.iter_mut().for_each(|word| {
            *word = format!("{}{}", transform_prefix, word);
        });
    }
    if opts.transform_suffix.is_some() {
        let transform_suffix = opts.transform_suffix.clone().unwrap();
        words.iter_mut().for_each(|word| {
            *word = format!("{}{}", word, transform_suffix);
        });
    }
    if opts.transform_capitalize {
        words.iter_mut().for_each(|word| {
            *word = word.to_lowercase();
            let mut chars = word.chars();
            if let Some(first_char) = chars.next() {
                *word = format!("{}{}", first_char.to_uppercase(), chars.as_str());
            }
        });
    }
}

pub fn is_response_filtered(opts: &Opts, res_text: &str, status_code: u16, time: u16) -> bool {
    if opts.filter_time.is_some() {
        let filter_time = opts.filter_time.clone().unwrap();
        let parsed_filter_time = parse_range_input(&filter_time).unwrap();
        if !check_range(&parsed_filter_time, time as usize) {
            return false;
        }
    }
    if opts.filter_status_code.is_some() {
        let filter_status_code = opts.filter_status_code.clone().unwrap();
        let parsed_filter_status_code = parse_range_input(&filter_status_code).unwrap();
        if !check_range(&parsed_filter_status_code, status_code as usize) {
            return false;
        }
    }
    if opts.filter_contains.is_some() {
        let filter_contains = opts.filter_contains.clone().unwrap();
        if !res_text.contains(&filter_contains) {
            return false;
        }
    }
    if opts.filter_starts_with.is_some() {
        let filter_starts_with = opts.filter_starts_with.clone().unwrap();
        if !res_text.starts_with(&filter_starts_with) {
            return false;
        }
    }
    if opts.filter_ends_with.is_some() {
        let filter_ends_with = opts.filter_ends_with.clone().unwrap();
        if !res_text.ends_with(&filter_ends_with) {
            return false;
        }
    }
    if opts.filter_regex.is_some() {
        let filter_regex = opts.filter_regex.clone().unwrap();
        let re = regex::Regex::new(&filter_regex).unwrap();
        if !re.is_match(&res_text) {
            return false;
        }
    }
    if opts.filter_length.is_some() {
        let filter_length = opts.filter_length.clone().unwrap();
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

/// Check if any response filter is set
pub fn should_filter(opts: &Opts) -> bool {
    return opts.filter_status_code.is_some()
        || opts.filter_contains.is_some()
        || opts.filter_starts_with.is_some()
        || opts.filter_ends_with.is_some()
        || opts.filter_regex.is_some()
        || opts.filter_length.is_some()
        || opts.filter_time.is_some();
}

pub fn save_to_file(
    opts: &Opts,
    root: Arc<Mutex<TreeNode<TreeData>>>,
    depth: Arc<Mutex<usize>>,
    tree: Arc<Mutex<Tree<TreeData>>>,
) -> Result<()> {
    let output = opts.output.clone().unwrap();
    let file_type = output.split(".").last().unwrap_or("json");
    let mut file = std::fs::File::create(opts.output.clone().unwrap())?;

    match file_type {
        "json" => {
            file.write_all(serde_json::to_string(&*root.lock())?.as_bytes())?;
            file.flush()?;
            Ok(())
        }
        "csv" => {
            let mut writer = csv::Writer::from_writer(file);
            let mut nodes = Vec::new();
            for depth in 0..*depth.lock() {
                nodes.append(&mut tree.lock().get_nodes_at_depth(depth));
            }
            for node in nodes {
                writer.serialize(node.lock().data.clone())?;
            }
            writer.flush()?;
            Ok(())
        }
        "md" => {
            let mut nodes = Vec::new();
            for depth in 0..*depth.lock() {
                nodes.append(&mut tree.lock().get_nodes_at_depth(depth));
            }
            for node in nodes {
                let data = node.lock().data.clone();
                let emoji = get_emoji_for_status_code(data.status_code);
                let url = data.url;
                let path = data.path;
                let depth = data.depth;
                let status_code = data.status_code;
                let line = format!(
                    "{}- [{} /{} {}]({})",
                    "  ".repeat(depth),
                    emoji,
                    path.trim_start_matches("/"),
                    if status_code == 0 {
                        "".to_string()
                    } else {
                        format!("({})", status_code)
                    },
                    url,
                );
                file.write_all(line.as_bytes())?;
                file.write_all(b"\n")?;
            }
            file.flush()?;
            Ok(())
        }
        _ => {
            let mut nodes = Vec::new();
            for depth in 0..*depth.lock() {
                nodes.append(&mut tree.lock().get_nodes_at_depth(depth));
            }
            for node in nodes {
                let data = node.lock().data.clone();
                file.write_all(data.url.as_bytes())?;
                file.write_all(b"\n")?;
            }
            file.flush()?;
            Ok(())
        }
    }
}
