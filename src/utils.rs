use anyhow::{bail, Context, Result};
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

pub fn parse_wordlists(wordlists: &Vec<String>) -> Result<Vec<String>> {
    let mut wordlist = Vec::new();
    for wordlist_path in wordlists {
        let mut file = std::fs::File::open(wordlist_path).with_context(|| {
            format!(
                "Failed to open wordlist file: {}",
                wordlist_path.to_string().bold().red()
            )
        })?;
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
    Ok(wordlist)
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

pub fn apply_transformations(opts: &Opts, words: &mut Vec<String>) {
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

// Returns true if the response should be kept
pub fn is_response_filtered(opts: &Opts, res_text: &str, status_code: u16, time: u16) -> bool {
    for filter in &opts.filter {
        let not = filter.0.starts_with("!");
        match filter.0.trim_start_matches("!") {
            "time" => {
                let parsed_filter_time = parse_range_input(&filter.1).unwrap();
                if !check_range(&parsed_filter_time, time as usize) {
                    return not;
                } else {
                    return !not;
                }
            }
            "status" => {
                let parsed_filter_status_code = parse_range_input(&filter.1).unwrap();
                if !check_range(&parsed_filter_status_code, status_code as usize) {
                    return not;
                } else {
                    return !not;
                }
            }
            "contains" => {
                if !res_text.contains(&filter.1) {
                    return not;
                } else {
                    return !not;
                }
            }
            "starts" => {
                if !res_text.starts_with(&filter.1) {
                    return not;
                } else {
                    return !not;
                }
            }
            "ends" => {
                if !res_text.ends_with(&filter.1) {
                    return not;
                } else {
                    return !not;
                }
            }
            "regex" => {
                let re = regex::Regex::new(&filter.1).unwrap();
                if !re.is_match(res_text) {
                    return not;
                } else {
                    return !not;
                }
            }
            "length" => {
                let parsed_filter_length = parse_range_input(&filter.1).unwrap();
                if !check_range(&parsed_filter_length, res_text.len()) {
                    return not;
                } else {
                    return !not;
                }
            }
            "hash" => {
                let hash = md5::compute(res_text);
                if !filter.1.contains(&format!("{:x}", hash)) {
                    return not;
                } else {
                    return !not;
                }
            }
            _ => {}
        }
    }

    false
}

pub fn check_range(ranges: &Vec<(usize, usize)>, num: usize) -> bool {
    for range in ranges {
        if num >= range.0 && num <= range.1 {
            return true;
        }
    }
    false
}

pub fn parse_range_input(s: &str) -> Result<Vec<(usize, usize)>> {
    let mut ranges = Vec::new();
    let parts = s.split(",").collect::<Vec<_>>();
    for part in parts {
        if part.is_empty() {
            continue;
        }
        if part.starts_with(">") {
            let num = part[1..].parse::<usize>();
            match num {
                Ok(num) => ranges.push((num + 1, usize::MAX)),
                Err(_) => bail!("Invalid range"),
            }
        } else if part.starts_with("<") {
            let num = part[1..].parse::<usize>();
            match num {
                Ok(num) => ranges.push((0, num - 1)),
                Err(_) => bail!("Invalid range"),
            }
        } else {
            let part = part.trim();
            let parts = part.split("-").collect::<Vec<_>>();
            if parts.len() == 1 {
                let num = parts[0].parse::<usize>();
                match num {
                    Ok(num) => ranges.push((num, num)),
                    Err(_) => bail!("Invalid range"),
                }
            } else if parts.len() == 2 {
                let num1 = parts[0].parse::<usize>();
                let num2 = parts[1].parse::<usize>();
                match (num1, num2) {
                    (Ok(num1), Ok(num2)) => ranges.push((num1, num2)),
                    _ => bail!("Invalid range"),
                }
            } else {
                bail!("Invalid range")
            }
        }
    }
    Ok(ranges)
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
