use std::path::PathBuf;

use crate::{
    constants::THREADS_PER_CORE,
    types::EngineMode,
    utils::types::{HTTPMethod, IntRange},
};
use clap::builder::EnumValueParser;
use clap::Parser;
use cowstr::CowStr;
use dashmap::DashSet as HashSet;
use merge::Merge;
use parse::{
    parse_filter, parse_keyed_key_or_keyval, parse_keyed_keyval, parse_url, parse_wordlist,
};
use serde::Deserialize;
use url::Url;

pub mod help;
pub mod parse;
pub mod utils;

#[derive(Debug, Parser, Clone, Merge, Deserialize)]
#[clap(version = utils::version(), long_version = utils::long_version(), disable_help_flag = true, help_template = "{all-args}")]
pub struct Opts {
    #[clap(short, hide = true)]
    #[merge(skip)]
    pub help: bool,

    #[clap(long = "help", hide = true)]
    #[merge(skip)]
    pub help_long: bool,

    /// URL to scan
    #[clap(value_parser = parse_url, required_unless_present_any(["list_filters", "list_transforms", "help", "help_long", "list"]))]
    #[merge(strategy = merge_overwrite)]
    pub url: Option<Url>,

    /// Wordlist file(s) to use, `path[:key]`
    #[clap(value_parser = parse_wordlist, required_unless_present_any(["list_filters", "list_transforms", "help", "help_long", "resume", "list"]))]
    #[merge(strategy = merge::vec::append)]
    pub wordlists: Vec<(String, String)>,

    /// Number of threads to use, defaults to `num_cores * 5`
    #[clap(short = 'T', long, default_value_t = num_cpus::get() * THREADS_PER_CORE, help_heading = "Execution Control")]
    #[merge(strategy = merge_overwrite)]
    pub threads: usize,

    /// Request rate limit in requests per second
    #[clap(long, visible_alias = "rate", help_heading = "Execution Control")]
    pub throttle: Option<u64>,

    /// Fuzzing mode
    #[clap(short, long, default_value = "recursive", value_parser = EnumValueParser::<EngineMode>::new(), help_heading = "Execution Control")]
    #[merge(strategy = merge_overwrite)]
    pub mode: EngineMode,

    /// Only use HTTP/1
    #[clap(long, help_heading = "Execution Control")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub http1: bool,

    /// Only use HTTP/2
    #[clap(long, help_heading = "Execution Control")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub http2: bool,

    /// Maximum depth in recursive mode
    #[clap(short, long, default_value = "0", help_heading = "Scanning Behavior")]
    #[merge(strategy = merge_overwrite)]
    pub depth: usize,

    /// Maximum retries for failed requests
    #[clap(
        short,
        long,
        default_value = "3",
        visible_alias = "retry",
        help_heading = "Scanning Behavior"
    )]
    #[merge(strategy = merge_overwrite)]
    pub retries: usize,

    /// What status codes to retry on
    #[clap(long, visible_alias = "retry-on", help_heading = "Scanning Behavior")]
    #[merge(strategy = merge_overwrite)]
    pub retry_codes: Vec<IntRange<u16>>,

    /// Force the scan, even if the target is unreachable
    #[clap(long, help_heading = "Scanning Behavior")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub force: bool,

    /// Force the recursion, even if the URL is not detected as a directory
    #[clap(long, visible_alias = "fr", help_heading = "Scanning Behavior")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub force_recursion: bool,

    /// HTTP method to use
    #[clap(short = 'X', long, value_parser = EnumValueParser::<HTTPMethod>::new(), default_value = "GET", help_heading = "Scanning Behavior")]
    #[merge(strategy = merge_overwrite)]
    pub method: HTTPMethod,

    /// Headers to send with the request, `name:value`
    #[clap(short = 'H', long, value_delimiter = ';', value_name = "HEADER", value_parser = parse_keyed_keyval, help_heading = "Scanning Behavior")]
    #[merge(strategy = merge::vec::append)]
    pub headers: Vec<(HashSet<String>, String, String)>,

    /// Extra information to display on hits
    #[clap(short, long, value_delimiter = ',', help_heading = "Output & Display")]
    #[merge(strategy = merge::vec::append)]
    pub show: Vec<String>,

    /// Save responses to a file, supported: `json`, `csv`, `txt`, `md`
    #[clap(short, long, help_heading = "Output & Display")]
    pub output: Option<PathBuf>,

    /// Ring the terminal bell on hits
    #[clap(
        long,
        visible_alias = "ding",
        visible_alias = "dong",
        help_heading = "Output & Display"
    )]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub bell: bool,

    /// List of filters to apply to responses, see `--list-filters`
    #[clap(short, long, value_parser = parse_filter, visible_alias = "filter", value_name ="EXPR", help_heading = "Filters & Transforms")]
    #[merge(strategy = merge::vec::append)]
    pub filters: Vec<String>,

    /// List of transformations to apply to wordlists, see `--list-transforms`
    #[clap(short, long, value_parser = parse_keyed_key_or_keyval, value_delimiter = ';', visible_alias = "transform", value_name = "TRANSFORM", help_heading = "Filters & Transforms")]
    #[merge(strategy = merge::vec::append)]
    pub transforms: Vec<(HashSet<String>, String, Option<String>)>,

    /// Wordlist filters, see `--list-filters`
    #[clap(
        short,
        long,
        visible_alias = "wf",
        value_name = "EXPR",
        help_heading = "Filters & Transforms"
    )]
    pub wordlist_filter: Option<String>,

    /// Resume from previous session
    #[clap(long, help_heading = "Session Management")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub resume: bool,

    /// Don't save state on `Ctrl+C`
    #[clap(long, help_heading = "Session Management")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub no_save: bool,

    /// Load configuration from a file, merges with command line arguments
    #[merge(skip)]
    #[clap(short, long, help_heading = "Miscellaneous")]
    pub config: Option<PathBuf>,

    /// List available filters (wordlist and response)
    #[merge(skip)]
    #[clap(long, help_heading = "Miscellaneous")]
    pub list_filters: bool,

    /// List available wordlist transforms
    #[merge(skip)]
    #[clap(long, help_heading = "Miscellaneous")]
    pub list_transforms: bool,

    /// List both available filters and wordlist transforms
    #[merge(skip)]
    #[clap(long, help_heading = "Miscellaneous")]
    pub list: bool,
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
