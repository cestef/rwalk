use clap::Parser;
use reqwest::Url;

#[derive(Debug, Parser, Clone)]
pub struct Opts {
    pub url: Url,
    pub wordlists: Vec<String>,
    #[clap(short, long)]
    pub threads: Option<usize>,
}
