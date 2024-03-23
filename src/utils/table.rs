use std::collections::HashMap;

use crate::{
    cli::{
        helpers::{KeyOrKeyVal, KeyVal},
        opts::Opts,
    },
    runner::wordlists::ParsedWordlist,
};
use colored::{Colorize, CustomColor};
use tabled::{
    builder::Builder,
    settings::{Alignment, Style},
};

use super::{color_for_status_code, structs::Mode};

pub fn build_opts_table(
    opts: &Opts,
    words: &HashMap<String, ParsedWordlist>,
    mode: &Mode,
    threads: usize,
    url: String,
) -> String {
    let mut builder = Builder::default();

    let mut filters_builder = Builder::default();
    filters_builder.push_record(vec!["Filter", "Value"]);
    for filter in &opts.filter {
        match filter {
            KeyVal(k, v) if k == "status" => {
                let mut out = String::new();
                for status in v.split(',') {
                    let mut status = status.to_string();
                    if status.contains('-') {
                        status = status
                            .split('-')
                            .map(|x| match x.parse::<u16>() {
                                Ok(x) => color_for_status_code(x.to_string(), x),
                                Err(_) => x.to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join("-")
                            .to_string();
                    } else if let Some(stripped) = status.strip_prefix('>') {
                        status = ">".to_string()
                            + &color_for_status_code(
                                stripped.to_string(),
                                stripped.parse().unwrap_or_default(),
                            );
                    } else if let Some(stripped) = status.strip_prefix('<') {
                        status = "<".to_string()
                            + &color_for_status_code(
                                stripped.to_string(),
                                stripped.parse().unwrap_or_default(),
                            );
                    } else {
                        status = color_for_status_code(
                            status.to_string(),
                            status.parse().unwrap_or_default(),
                        );
                    }
                    out.push_str(&status);
                    out.push_str(", ");
                }

                filters_builder.push_record(vec![k, &out.trim_end_matches(", ").to_string()]);
            }
            KeyVal(k, v) => filters_builder.push_record(vec![k, &v.bold().to_string()]),
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

    builder.push_record(vec![
        "URL",
        &url.trim_end_matches('/')
            .to_string()
            .bold()
            .blue()
            .to_string(),
    ]);

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
            "{} for {} CPUs",
            match threads / num_cpus::get() {
                0..=10 => threads.to_string().bold().green(),
                11..=20 => threads.to_string().bold().yellow(),
                _ => threads.to_string().bold().red(),
            },
            num_cpus::get().to_string().bold()
        ),
    ]);

    builder
        .build()
        .with(Style::modern_rounded())
        .with(Alignment::center_vertical())
        .to_string()
}
