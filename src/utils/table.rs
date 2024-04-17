use std::collections::HashMap;

use crate::{
    cli::{
        helpers::{KeyOrKeyVal, KeyVal},
        opts::Opts,
    },
    runner::wordlists::ParsedWordlist,
    utils::{
        display::{display_range, display_range_status},
        is_range,
    },
};
use colored::{Colorize, CustomColor};
use log::{info, warn};
use tabled::{
    builder::Builder,
    settings::{Alignment, Style},
};

use super::structs::{FuzzMatch, Mode};

/// Builds the options table printed in the CLI
pub fn build_opts_table(
    opts: &Opts,
    words: &HashMap<String, ParsedWordlist>,
    mode: &Mode,
    threads: usize,
    url: String,
    fuzz_matches: &[FuzzMatch],
) -> String {
    let mut builder = Builder::default();

    let mut filters_builder = Builder::default();
    filters_builder.push_record(vec!["Depth", "Filter", "Value"]);
    for filter in &opts.filter {
        match filter.clone() {
            KeyVal(mut k, v) if k == "status" => {
                let out = v
                    .split(',')
                    .map(|status| display_range_status(status.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                let filter_depth = if k.starts_with('[') {
                    let start_index = k.find('[').unwrap();
                    let end_index = k.find(']').unwrap();
                    let depth = k[start_index + 1..end_index].parse::<usize>();
                    k = k[end_index + 1..].to_string();
                    if let Ok(d) = depth {
                        Some(d)
                    } else {
                        warn!("Invalid depth filter: {}", depth.unwrap_err());
                        None
                    }
                } else {
                    None
                };
                filters_builder.push_record(vec![
                    filter_depth.map_or("*".to_string(), |x| x.to_string()),
                    k,
                    out.trim_end_matches(", ").to_string(),
                ]);
            }
            KeyVal(mut k, v) => {
                let filter_depth = if k.starts_with('[') {
                    let start_index = k.find('[').unwrap();
                    let end_index = k.find(']').unwrap();
                    let depth = k[start_index + 1..end_index].parse::<usize>();
                    k = k[end_index + 1..].to_string();
                    if let Ok(d) = depth {
                        Some(d)
                    } else {
                        warn!("Invalid depth filter: {}", depth.unwrap_err());
                        None
                    }
                } else {
                    None
                };
                // Try to parse the value as a range
                let is_range = is_range(&v);
                let v = if is_range {
                    display_range(v.to_string())
                } else {
                    v
                };

                filters_builder.push_record(vec![
                    filter_depth.map_or("*".to_string(), |x| x.to_string()),
                    k,
                    v,
                ]);
            }
        }
    }

    builder.push_record(vec![
        "Filters",
        &filters_builder
            .build()
            .with(Style::modern_rounded())
            .to_string(),
    ]);

    if !opts.show.is_empty() {
        builder.push_record(vec![
            "Show",
            &opts
                .show
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ")
                .bold()
                .to_string(),
        ]);
    }

    if opts.depth.is_some() {
        builder.push_record(vec![
            "Depth",
            &opts.depth.as_ref().unwrap().to_string().bold(),
        ]);
    }

    if !opts.transform.is_empty() {
        let mut wordlist_filters_builder = Builder::default();
        wordlist_filters_builder.push_record(vec!["Wordlist", "Filter", "Value"]);
        for KeyOrKeyVal(k, v) in &opts.transform {
            // can be [key]filter=value or filter=value
            let (key, filter) = if k.starts_with('[') {
                let mut split = k.split(']');
                let key = split.next().unwrap().trim_start_matches('[');
                let filter = split.next().unwrap_or("").trim_start_matches(' ');
                (key, filter.to_string())
            } else {
                ("*", k.to_string())
            };

            wordlist_filters_builder.push_record(vec![
                key,
                &filter,
                &(v.as_ref().unwrap_or(&"".to_string()).bold().to_string()),
            ]);
        }
        builder.push_record(vec![
            "Wordlist Filters",
            &wordlist_filters_builder
                .build()
                .with(Style::modern_rounded())
                .to_string(),
        ]);
    }

    if !opts.wordlist_filter.is_empty() {
        let mut wordlist_filters_builder = Builder::default();
        wordlist_filters_builder.push_record(vec!["Wordlist", "Filter", "Value"]);
        for KeyVal(k, v) in &opts.wordlist_filter {
            // can be [key]filter=value or filter=value
            let (key, filter) = if k.starts_with('[') {
                let mut split = k.split(']');
                let key = split.next().unwrap().trim_start_matches('[');
                let filter = split.next().unwrap_or("").trim_start_matches(' ');
                (key, filter.to_string())
            } else {
                ("*", k.to_string())
            };

            wordlist_filters_builder.push_record(vec![key, &filter, &v.bold().to_string()]);
        }
        builder.push_record(vec![
            "Wordlist Filters",
            &wordlist_filters_builder
                .build()
                .with(Style::modern_rounded())
                .to_string(),
        ]);
    }

    let mut url = url.trim_end_matches('/').to_string();

    // Only color the url parts that have been matched with fuzz_matches

    fn color_n(s: String, n: usize) -> String {
        match n % 5 {
            0 => s.bold().green().to_string(),
            1 => s.bold().yellow().to_string(),
            2 => s.bold().red().to_string(),
            3 => s.bold().cyan().to_string(),
            _ => s.bold().magenta().to_string(),
        }
    }

    let grouped_matches = fuzz_matches
        .iter()
        .fold(HashMap::<String, Vec<&FuzzMatch>>::new(), |mut acc, x| {
            acc.entry(x.content.clone()).or_default().push(x);
            acc
        })
        .into_iter()
        .collect::<Vec<_>>();

    for (i, matches) in grouped_matches.iter().enumerate() {
        for fuzz_match in &matches.1 {
            url = url.replace(
                &fuzz_match.content,
                &color_n(fuzz_match.content.to_string(), i),
            );
        }
    }

    builder.push_record(vec!["URL", &url]);

    let mut wordlists_builder = Builder::default();
    wordlists_builder.push_record(vec!["Path", "Key", "Size"]);
    for (k, v) in words {
        wordlists_builder.push_record(vec![
            &v.path.bold().blue().to_string(),
            &k.bold().to_string(),
            &match v.words.len() {
                0..=1000 => v.words.len().to_string().bold().green().to_string(),
                1001..=10000 => v.words.len().to_string().bold().yellow().to_string(),
                10001..=100000 => v
                    .words
                    .len()
                    .to_string()
                    .bold()
                    // Orange
                    .custom_color(CustomColor::new(208, 104, 63))
                    .to_string(),
                _ => v.words.len().to_string().bold().red().to_string(),
            },
        ]);
    }

    builder.push_record(vec![
        "Wordlists",
        &wordlists_builder
            .build()
            .with(Style::modern_rounded())
            .to_string(),
    ]);

    builder.push_record(vec!["Mode", &mode.to_string().bold().to_string()]);

    builder.push_record(vec![
        "Threads",
        &format!(
            "{} for {} core{}",
            match threads / num_cpus::get() {
                0..=10 => threads.to_string().bold().green(),
                11..=20 => threads.to_string().bold().yellow(),
                _ => threads.to_string().bold().red(),
            },
            num_cpus::get().to_string().bold(),
            if num_cpus::get() > 1 { "s" } else { "" }
        ),
    ]);

    builder
        .build()
        .with(Style::modern_rounded())
        .with(Alignment::center_vertical())
        .to_string()
}
