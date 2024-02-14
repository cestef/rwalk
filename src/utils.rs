use anyhow::{bail, Result};
use colored::Colorize;
use parking_lot::Mutex;
use std::{io::Write, sync::Arc};

use crate::{
    cli::Opts,
    constants::BANNER_STR,
    tree::{Tree, TreeData, TreeNode},
};

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

// Returns true if the response should be kept
pub fn is_response_filtered(opts: &Opts, res_text: &str, status_code: u16, time: u16) -> bool {
    let mut outs: Vec<bool> = Vec::new();

    let filters = if opts.filter.iter().any(|e| e.0 == "status") {
        opts.filter.clone()
    } else {
        let mut filters = opts.filter.clone();
        filters.push((
            "status".to_string(),
            "200-299,301,302,307,401,403,405,500".to_string(),
        ));
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
