#![allow(dead_code, unused_imports)]

use anyhow::{Context, Ok, Result};
use async_recursion::async_recursion;
use clap::Parser;
use cli::Args;
use colored::*;
use indicatif::{HumanDuration, ParallelProgressIterator, ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use log::log;
use ptree::item::StringItem;
use ptree::{print_tree, TreeBuilder, TreeItem};
use reqwest::redirect::Policy;
use std::borrow::Cow;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{
    fs::File,
    io::{BufReader, Read},
    time::Duration,
};
use stopwatch::Stopwatch;
use url::Url;

use crate::log::{log_error, log_success, log_verbose, log_warning};
use crate::tree::{tree_to_vec, PathTree, TreeTraverser};

mod cli;
mod crawler;
mod log;
mod manager;
mod tree;
mod wordlists;

lazy_static! {
    static ref ARGS: Args = Args::parse();
}

#[tokio::main]
async fn main() -> Result<()> {
    ctrlc::set_handler(move || {
        log(
            &format!("{} {}", "✖".red().bold(), "Aborted by user".red().bold()),
            true,
        );
        std::process::exit(1);
    })
    .expect("Error setting Ctrl-C handler");
    let fixed_host = if ARGS.host.starts_with("http://") || ARGS.host.starts_with("https://") {
        ARGS.host.clone()
    } else {
        format!("http://{}", ARGS.host.clone())
    };
    let parsed_host = Url::parse(&fixed_host).with_context(|| "Failed to parse host")?;

    // Check if host is reachable
    let spinner = indicatif::ProgressBar::new_spinner();
    {
        spinner.set_message("🔎 Checking host");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(ARGS.timeout))
            .redirect(Policy::none())
            .build()
            .unwrap();
        let res = client.get(parsed_host.clone()).send().await;
        if let std::result::Result::Err(err) = res {
            spinner.finish_and_clear();
            println!(
                "{} {}",
                "✖".red().bold(),
                format!("Failed to connect to host: {}", err).red()
            );
            std::process::exit(1);
        }
    }

    let mut wordlist_parser = wordlists::WordlistParser::new(ARGS.wordlists.clone());
    let watch = Stopwatch::start_new();
    spinner.set_message("📁 Reading files");
    wordlist_parser.read()?;
    spinner.set_message("🔀 Merging files");
    wordlist_parser.merge()?;

    let mut words = wordlist_parser.contents.clone();

    if ARGS.wordlists.len() > 1 || ARGS.case_insensitive {
        spinner.set_message("🧹 Removing duplicates");
        words.sort_unstable();
        let old_len = words.len();
        words.dedup();
        spinner.finish_and_clear();

        let diff = old_len - words.len();

        println!(
            "🧹 Removed {} duplicates ({:.2}%)",
            diff.to_string().red().bold(),
            diff as f64 / old_len as f64 * 100.0
        );
    } else {
        spinner.finish_and_clear();
    }

    let size = words.iter().map(|s| s.len()).sum::<usize>();

    println!(
        "📜 Parsed {} words in {}ms {}",
        words.len().to_string().green().bold(),
        watch.elapsed_ms().to_string().bold(),
        format!("(~{:.2}MB)", size as f64 / 1024.0 / 1024.0).dimmed()
    );

    let watch = Stopwatch::start_new();
    let cpus = num_cpus::get();
    let threads = match ARGS.threads {
        0 => cpus * 10,
        _ => ARGS.threads,
    };
    println!(
        "🧵 Using {} threads {}",
        threads.to_string().bold(),
        format!("({}/CPU)", threads / cpus).dimmed()
    );

    let progress = indicatif::ProgressBar::new(words.len() as u64).with_style(
        ProgressStyle::default_bar()
            .template("{msg} [{pos:>2}/{len:2}] ETA: ~{eta:3} ({per_sec})")
            .unwrap(),
    );
    progress.enable_steady_tick(Duration::from_millis(100));

    progress.set_message("🔎 Crawling");
    let words_len = words.len();
    let manager = manager::CrawlerManager::new(
        parsed_host.clone(),
        words.clone(),
        threads,
        progress.clone(),
    );

    let mut traverser = TreeTraverser::new(
        parsed_host.clone(),
        words.clone(),
        threads,
        progress.clone(),
        PathTree {
            name: parsed_host
                .path()
                .to_string()
                .trim_end_matches("/")
                .to_string(),
            children: manager
                .run()
                .await?
                .iter()
                .map(|urls| PathTree {
                    // name: Url::parse(s).unwrap().path().to_string(),
                    name: Url::parse(&urls[0]).unwrap().path().to_string(),
                    children: Vec::new(),
                })
                .collect::<Vec<PathTree>>(),
        },
        ARGS.depth,
    );

    traverser.traverse().await;

    progress.finish_and_clear();

    let mut found = Vec::new();

    tree_to_vec(&traverser.tree, &mut found);

    println!(
        "🔎 Discovered {} path{} in {} {}",
        found.len().to_string().green().bold(),
        if found.len() == 1 { "" } else { "s" },
        HumanDuration(watch.elapsed()).to_string().bold(),
        format!(
            "(μ~{:.2} paths/s)",
            words_len as f64 / watch.elapsed_ms() as f64 * 1000.0
        )
        .dimmed()
    );

    found.sort_unstable();

    let old_len = found.len();
    found.dedup();
    let diff = old_len - found.len();
    if diff > 0 {
        println!(
            "🧹 Removed {} duplicates from found paths ({:.2}%)",
            diff.to_string().red().bold(),
            diff as f64 / old_len as f64 * 100.0
        );
    }

    let mut output = match &ARGS.output {
        Some(path) => {
            let file = File::create(path).with_context(|| "Failed to create output file")?;
            Some(file)
        }
        None => None,
    };

    match output {
        Some(ref mut file) => {
            for path in &found {
                writeln!(file, "{}", parsed_host.join(path).unwrap())?;
            }
            println!(
                "📝 Wrote paths to {}",
                ARGS.output.as_ref().unwrap().to_str().unwrap().bold()
            );
        }
        None => {
            print_tree(&traverser.tree)?;
        }
    }

    Ok(())
}
