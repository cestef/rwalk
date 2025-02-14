use std::path::PathBuf;

use clap::Parser;
use cowstr::CowStr;
use dashmap::DashSet as HashSet;
use merge::Merge;
use parse::{parse_filter, parse_keyed_key_or_keyval, parse_url, parse_wordlist};
use serde::Deserialize;
use url::Url;

pub mod parse;
pub mod utils;

use crate::{constants::THREADS_PER_CORE, types::EngineMode};

#[derive(Debug, Parser, Clone, Merge, Deserialize)]
#[clap(version = utils::version(), long_version = utils::long_version())]
pub struct Opts {
    #[clap(value_parser = parse_url)]
    #[merge(strategy = merge_overwrite)]
    pub url: Url,
    /// Wordlist file(s) to use, path[:key]
    #[clap(value_parser = parse_wordlist)]
    #[merge(strategy = merge::vec::append)]
    pub wordlists: Vec<(String, String)>,
    /// Number of threads to use, defaults to num. of cores * 10
    #[clap(short = 'T', long, default_value_t = num_cpus::get() * THREADS_PER_CORE)]
    #[merge(strategy = merge_overwrite)]
    pub threads: usize,
    /// List of filters to apply to responses, name:value
    #[clap(short, long, visible_alias = "filter", value_parser = parse_filter)]
    #[merge(strategy = merge::vec::append)]
    pub filters: Vec<String>,
    /// List of transformations to apply to wordlists, [key:]name[:value]
    #[clap(short, long, value_parser = parse_keyed_key_or_keyval, value_delimiter = ';', visible_alias = "transform")]
    #[merge(strategy = merge::vec::append)]
    pub transforms: Vec<(HashSet<String>, String, Option<String>)>,
    /// Fuzzing mode, one of: recursive (r), template (t)
    #[clap(short, long, default_value = "recursive")]
    #[merge(strategy = merge_overwrite)]
    pub mode: EngineMode,
    /// Request rate limit in requests per second
    #[clap(long, visible_alias = "rps")]
    pub throttle: Option<u64>,
    /// Maximum depth in recursive mode
    #[clap(short, long, default_value = "3")]
    #[merge(strategy = merge_overwrite)]
    pub depth: usize,
    /// Maximum retries for failed requests
    #[clap(short, long, default_value = "3", visible_alias = "retry")]
    #[merge(strategy = merge_overwrite)]
    pub retries: usize,
    /// Only use HTTP/1
    #[clap(long)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub http1: bool,
    /// Only use HTTP/2
    #[clap(long)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub http2: bool,
    /// Resume from previous session
    #[clap(long)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub resume: bool,
    /// Don't save state on Ctrl+C
    #[clap(long)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub no_save: bool,
    /// Extra information to display on hits
    #[clap(short, long)]
    #[merge(strategy = merge::vec::append)]
    pub show: Vec<String>,
    /// Wordlist filters
    #[clap(short, long, visible_alias = "wf")]
    #[merge(strategy = merge::vec::append)]
    pub wordlist_filters: Vec<String>,
    /// Force the scan, even if the target is unreachable
    #[clap(long)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub force: bool,

    #[merge(skip)]
    #[clap(short, long)]
    pub config: Option<PathBuf>,
}

fn display_wordlists(wordlists: &Vec<(CowStr, CowStr)>) -> String {
    wordlists
        .iter()
        .map(|(path, key)| format!("{}:{}", path, key))
        .collect::<Vec<String>>()
        .join(", ")
}

fn merge_overwrite<T>(a: &mut T, b: T) {
    *a = b;
}
