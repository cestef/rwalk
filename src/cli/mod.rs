use clap::Parser;
use dashmap::DashSet as HashSet;
use parse::{parse_keyed_key_or_keyval, parse_keyed_keyval, parse_wordlist};
use url::Url;

pub mod parse;

use crate::{constants::THREADS_PER_CORE, types::EngineMode};

#[derive(Debug, Parser, Clone)]
pub struct Opts {
    pub url: Url,
    #[clap(value_parser = parse_wordlist)]
    pub wordlists: Vec<(String, String)>,
    #[clap(short = 'T', long, default_value_t = num_cpus::get() * THREADS_PER_CORE)]
    pub threads: usize,
    #[clap(short, long, value_parser = parse_keyed_keyval, value_delimiter = ';', visible_alias = "filter")]
    pub filters: Vec<(String, String)>,
    #[clap(short, long, value_parser = parse_keyed_key_or_keyval, value_delimiter = ';', visible_alias = "transform")]
    pub transforms: Vec<(HashSet<String>, String, Option<String>)>,
    #[clap(short, long, default_value = "recursive")]
    pub mode: EngineMode,
}
