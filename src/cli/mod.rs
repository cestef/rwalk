use clap::Parser;
use miette::Result;
use url::Url;

use crate::constants::THREADS_PER_CORE;

#[derive(Debug, Parser, Clone)]
pub struct Opts {
    pub url: Url,
    pub wordlists: Vec<String>,
    #[clap(short, long, default_value_t = num_cpus::get() * THREADS_PER_CORE)]
    pub threads: usize,
    #[clap(short, long, value_parser = parse_filter, value_delimiter = ';')]
    pub filters: Vec<(String, String)>,
}

// key:value
fn parse_filter(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(miette::miette!("Invalid filter format: {}", s));
    }
    Ok((parts[0].to_lowercase(), parts[1].to_string()))
}
