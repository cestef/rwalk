use clap::Parser;
use miette::Result;
use url::Url;

use crate::constants::THREADS_PER_CORE;

#[derive(Debug, Parser, Clone)]
pub struct Opts {
    pub url: Url,
    pub wordlists: Vec<String>,
    #[clap(short = 'T', long, default_value_t = num_cpus::get() * THREADS_PER_CORE)]
    pub threads: usize,
    #[clap(short, long, value_parser = parse_filter, value_delimiter = ';', visible_alias = "filter")]
    pub filters: Vec<(String, String)>,
    #[clap(short, long, value_parser = parse_transform, value_delimiter = ';', visible_alias = "transform")]
    pub transforms: Vec<(String, Option<String>)>,
}

// key:value
fn parse_filter(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(miette::miette!("Invalid filter format: {}", s));
    }
    Ok((parts[0].to_lowercase(), parts[1].to_string()))
}

// key[:value]
fn parse_transform(s: &str) -> Result<(String, Option<String>)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() > 2 {
        return Err(miette::miette!("Invalid transform format: {}", s));
    }
    Ok((parts[0].to_lowercase(), parts.get(1).map(|s| s.to_string())))
}
