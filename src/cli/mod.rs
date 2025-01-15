use clap::Parser;
use dashmap::DashSet as HashSet;
use parse::{parse_keyed_key_or_keyval, parse_keyval, parse_wordlist};
use url::Url;

pub mod parse;

use crate::{
    constants::THREADS_PER_CORE,
    types::EngineMode,
    utils::constants::{DEFAULT_THROTTLE_ERROR_THRESHOLD, DEFAULT_THROTTLE_WINDOW_SIZE_MILLIS},
};

#[derive(Debug, Parser, Clone)]
pub struct Opts {
    pub url: Url,
    /// Wordlist file(s) to use, [key:]path
    #[clap(value_parser = parse_wordlist)]
    pub wordlists: Vec<(String, String)>,
    /// Number of threads to use, defaults to num. of cores * 10
    #[clap(short = 'T', long, default_value_t = num_cpus::get() * THREADS_PER_CORE)]
    pub threads: usize,
    /// List of filters to apply to responses, name:value
    #[clap(short, long, value_parser = parse_keyval, value_delimiter = ';', visible_alias = "filter")]
    pub filters: Vec<(String, String)>,
    /// List of transformations to apply to wordlists, [key:]name[:value]
    #[clap(short, long, value_parser = parse_keyed_key_or_keyval, value_delimiter = ';', visible_alias = "transform")]
    pub transforms: Vec<(HashSet<String>, String, Option<String>)>,
    /// Fuzzing mode, one of: recursive (r), template (t)
    #[clap(short, long, default_value = "recursive")]
    pub mode: EngineMode,
    /// Request rate limit in requests per second, [lower:]upper
    #[clap(long, value_parser = parse::parse_throttle, visible_alias = "rps")]
    pub throttle: Option<(u64, u64)>,
    /// Duration of the window in milliseconds to calculate error rate for throttling
    #[clap(short, long, default_value_t = DEFAULT_THROTTLE_WINDOW_SIZE_MILLIS)]
    pub window: u64,
    /// Error rate threshold for throttling
    #[clap(long, default_value_t = DEFAULT_THROTTLE_ERROR_THRESHOLD, visible_alias = "et")]
    pub error_threshold: f64,
    /// Maximum depth in recursive mode
    #[clap(short, long, default_value = "3")]
    pub depth: usize,
    /// Maximum retries for failed requests
    #[clap(short, long, default_value = "3", visible_alias = "retry")]
    pub retries: usize,
    /// Only use HTTP/1
    #[clap(long)]
    pub http1: bool,
    /// Only use HTTP/2
    #[clap(long)]
    pub http2: bool,
}
