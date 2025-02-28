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

use crate::{constants::THREADS_PER_CORE, types::EngineMode, utils::types::IntRange};
use clap::builder::EnumValueParser;
#[derive(Debug, Parser, Clone, Merge, Deserialize)]
#[clap(version = utils::version(), long_version = utils::long_version(), disable_help_flag = true)]
pub struct Opts {
    /// Show this help message
    #[clap(short, long)]
    #[merge(skip)]
    pub help: bool,

    #[clap(value_parser = parse_url, required_unless_present_any(["list_filters", "list_transforms", "help"]))]
    #[merge(strategy = merge_overwrite)]
    pub url: Option<Url>,
    /// Wordlist file(s) to use, `path[:key]`
    #[clap(value_parser = parse_wordlist, required_unless_present_any(["list_filters", "list_transforms", "help"]))]
    #[merge(strategy = merge::vec::append)]
    pub wordlists: Vec<(String, String)>,
    /// Number of threads to use, defaults to `num_cores * 10`
    #[clap(short = 'T', long, default_value_t = num_cpus::get() * THREADS_PER_CORE)]
    #[merge(strategy = merge_overwrite)]
    pub threads: usize,
    /// List of filters to apply to responses, see `--list-filters`
    #[clap(short, long, value_parser = parse_filter, visible_alias = "filter", value_name ="EXPR")]
    #[merge(strategy = merge::vec::append)]
    pub filters: Vec<String>,
    /// List of transformations to apply to wordlists, see `--list-transforms`
    #[clap(short, long, value_parser = parse_keyed_key_or_keyval, value_delimiter = ';', visible_alias = "transform", value_name = "TRANSFORM")]
    #[merge(strategy = merge::vec::append)]
    pub transforms: Vec<(HashSet<String>, String, Option<String>)>,
    /// Fuzzing mode
    #[clap(short, long, default_value = "recursive", value_parser = EnumValueParser::<EngineMode>::new())]
    #[merge(strategy = merge_overwrite)]
    pub mode: EngineMode,
    /// Request rate limit in requests per second
    #[clap(long, visible_alias = "rate")]
    pub throttle: Option<u64>,
    /// Maximum depth in recursive mode
    #[clap(short, long, default_value = "0")]
    #[merge(strategy = merge_overwrite)]
    pub depth: usize,
    /// Maximum retries for failed requests
    #[clap(short, long, default_value = "3", visible_alias = "retry")]
    #[merge(strategy = merge_overwrite)]
    pub retries: usize,
    /// What status codes to retry on
    #[clap(long, visible_alias = "retry-on")]
    #[merge(strategy = merge_overwrite)]
    pub retry_codes: Vec<IntRange<u16>>,
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
    #[clap(short, long, value_delimiter = ',')]
    #[merge(strategy = merge::vec::append)]
    pub show: Vec<String>,
    /// Wordlist filters, see --list-filters
    #[clap(short, long, visible_alias = "wf", value_name = "EXPR")]
    pub wordlist_filter: Option<String>,
    /// Force the scan, even if the target is unreachable
    #[clap(long)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub force: bool,

    /// Force the recursion, even if the URL is not detected as a directory
    #[clap(long, visible_alias = "fr")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub force_recursion: bool,

    /// Save responses to a file, supported: json, csv, txt, md
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Load configuration from a file, merges with command line arguments
    #[merge(skip)]
    #[clap(short, long)]
    pub config: Option<PathBuf>,

    /// List available filters (wordlist and response)
    #[merge(skip)]
    #[clap(long)]
    pub list_filters: bool,

    /// List available wordlist transforms
    #[merge(skip)]
    #[clap(long)]
    pub list_transforms: bool,
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
