use color_eyre::eyre::{bail, Result};
use colored::{Colorize, CustomColor};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::{io::Write, sync::Arc};

use crate::cli::opts::Opts;
use crate::utils::tree::{Tree, TreeData, TreeNode};

use self::constants::DEFAULT_FILE_TYPE;

pub mod constants;
pub mod display;
pub mod extract;
pub mod logger;
pub mod scripting;
pub mod structs;
pub mod table;
pub mod tree;

pub static GIT_COMMIT_HASH: &str = env!("_GIT_INFO");

pub fn get_emoji_for_status_code_colored(status_code: u16) -> String {
    let emoji = get_emoji_for_status_code(status_code);
    color_for_status_code(emoji, status_code)
}

pub fn color_for_status_code(s: String, status_code: u16) -> String {
    match status_code {
        100..=199 => s.blue().to_string(),
        200..=299 => s.green().to_string(),
        300..=399 => s.yellow().to_string(),
        400..=499 => s.custom_color(CustomColor::new(255, 165, 0)).to_string(),
        500..=599 => s.red().to_string(),
        _ => s.to_string(),
    }
}

pub fn get_emoji_for_status_code(status_code: u16) -> String {
    match status_code {
        100..=199 => "ℹ".to_string(),
        200..=299 => "✓".to_string(),
        300..=399 => "⇝".to_string(),
        400..=403 => "✖".to_string(),
        500..=599 => "⚠".to_string(),
        _ => "⚠".to_string(),
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
    let parts = s.split(',').collect::<Vec<_>>();
    for part in parts {
        if part.is_empty() {
            continue;
        }
        // Greater than
        if let Some(stripped) = part.strip_prefix('>') {
            let num = stripped.parse::<usize>();
            match num {
                Ok(num) => ranges.push((num + 1, usize::MAX)),
                Err(_) => bail!("Invalid range"),
            }
        // Less than
        } else if let Some(stripped) = part.strip_prefix('<') {
            let num = stripped.parse::<usize>();
            match num {
                Ok(num) => ranges.push((0, num - 1)),
                Err(_) => bail!("Invalid range"),
            }
        } else {
            let part = part.trim();
            let parts = part.split('-').collect::<Vec<_>>();
            // Single number
            if parts.len() == 1 {
                let num = parts[0].parse::<usize>();
                match num {
                    Ok(num) => ranges.push((num, num)),
                    Err(_) => bail!("Invalid range"),
                }
            // Range
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

pub fn is_range(s: &str) -> bool {
    for part in s.split(',') {
        // >number, <number, number-number, number
        if let Some(stripped) = part.strip_prefix('>') {
            if stripped.parse::<usize>().is_ok() {
                return true;
            }
        } else if let Some(stripped) = part.strip_prefix('<') {
            if stripped.parse::<usize>().is_ok() {
                return true;
            }
        } else {
            let parts = part.split('-').collect::<Vec<_>>();
            if parts.len() == 1 {
                if parts[0].parse::<usize>().is_ok() {
                    return true;
                }
            } else if parts.len() == 2
                && parts[0].parse::<usize>().is_ok()
                && parts[1].parse::<usize>().is_ok()
            {
                return true;
            }
        }
    }
    false
}

pub fn init_panic() -> Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .capture_span_trace_by_default(false)
        .display_location_section(false)
        .display_env_section(false)
        .into_hooks();
    eyre_hook.install()?;
    std::panic::set_hook(Box::new(move |panic_info| {
        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, print_msg, Metadata};

            let meta = Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                .authors(env!("CARGO_PKG_AUTHORS").replace(':', ", "))
                .homepage(env!("CARGO_PKG_HOMEPAGE"));

            let file_path = handle_dump(&meta, panic_info);
            // prints human-panic message
            print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
            eprintln!("{}", panic_hook.panic_report(panic_info)); // prints color-eyre stack trace to stderr
        }
        let msg = format!("{}", panic_hook.panic_report(panic_info));
        log::error!("Error: {}", strip_ansi_escapes::strip_str(msg));

        #[cfg(debug_assertions)]
        {
            // Better Panic stacktrace that is only enabled when debugging.
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        std::process::exit(1);
    }));
    Ok(())
}
// Open the file in the default editor
pub fn open_file(file: &PathBuf) -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = std::process::Command::new(editor).arg(file).status()?;
    if !status.success() {
        bail!("Failed to open file in editor");
    }
    Ok(())
}

// Write the tree to a file (json, csv, md)
pub fn save_to_file(
    opts: &Opts,
    root: Arc<Mutex<TreeNode<TreeData>>>,
    depth: Arc<Mutex<usize>>,
    tree: Arc<Mutex<Tree<TreeData>>>,
) -> Result<()> {
    let output = opts.output.clone().unwrap();
    let file_type = output.split('.').next_back().unwrap_or(DEFAULT_FILE_TYPE);
    let mut file = std::fs::File::create(opts.output.clone().unwrap())?;

    match file_type {
        "json" => {
            let value = if opts.pretty {
                serde_json::to_string_pretty(&*root.lock())?
            } else {
                serde_json::to_string(&*root.lock())?
            };
            file.write_all(value.as_bytes())?;
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
                    path.trim_start_matches('/'),
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

pub fn version() -> String {
    let author = clap::crate_authors!();

    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let config_dir_path = dirs::home_dir()
        .map(|p| p.join(".config").join(clap::crate_name!()))
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
        .dimmed()
        .bold();
    let author = author.replace(':', ", ").dimmed().bold();
    let hash = GIT_COMMIT_HASH.bold();
    format!(
        "\
{hash}

Authors: {author}

Config directory: {config_dir_path}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_range() {
        assert!(check_range(&vec![(1, 2), (3, 4)], 1));
        assert!(check_range(&vec![(1, 2), (3, 4)], 2));
        assert!(check_range(&vec![(1, 2), (3, 4)], 3));
        assert!(check_range(&vec![(1, 2), (3, 4)], 4));
        assert!(!check_range(&vec![(1, 2), (3, 4)], 0));
        assert!(!check_range(&vec![(1, 2), (3, 4)], 5));
    }

    #[test]
    fn test_parse_range_input() {
        assert_eq!(parse_range_input("1-2").unwrap(), vec![(1, 2)]);
        assert_eq!(parse_range_input("1-2,3-4").unwrap(), vec![(1, 2), (3, 4)]);
        assert_eq!(parse_range_input("1,2").unwrap(), vec![(1, 1), (2, 2)]);
        assert_eq!(parse_range_input(">1").unwrap(), vec![(2, usize::MAX)]);
        assert_eq!(parse_range_input("<1").unwrap(), vec![(0, 0)]);
        assert_eq!(
            parse_range_input("1-2,>3").unwrap(),
            vec![(1, 2), (4, usize::MAX)]
        );

        assert!(parse_range_input("1-2,>3,4-").is_err());
    }

    #[test]
    fn test_get_emoji_for_status_code() {
        assert_eq!(get_emoji_for_status_code(200), "✓");
        assert_eq!(get_emoji_for_status_code(300), "⇝");
        assert_eq!(get_emoji_for_status_code(400), "✖");
        assert_eq!(get_emoji_for_status_code(500), "⚠");
        assert_eq!(get_emoji_for_status_code(0), "⚠");
    }
}
