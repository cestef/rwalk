use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Host to scan
    pub host: String,
    /// Optional port to scan
    #[clap(short, long)]
    pub port: Option<u16>,
    /// Path(s) to wordlist(s)
    #[clap(required = true)]
    pub wordlists: Vec<PathBuf>,
    /// Turn verbose information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// The number of threads to use
    /// Defaults to the number of logical cores * 10
    #[clap(short, long, default_value = "0")]
    pub threads: usize,
    /// Throttle requests to the host (in milliseconds)
    #[clap(short = 'T', long, default_value = "0")]
    pub throttle: u64,
    /// The number of retries to make
    #[clap(short = 'R', long, default_value = "0")]
    pub retries: u8,
    /// The number of redirects to follow
    #[clap(short, long, default_value = "0")]
    pub redirects: u8,
    /// The user agent to use
    #[clap(short, long)]
    pub user_agent: Option<String>,
    /// Maximum depth to crawl
    #[clap(short, long, default_value = "0")]
    pub depth: u8,
    /// Timeout for requests (in seconds)
    #[clap(short, long, default_value = "5")]
    pub timeout: u64,
    /// The output file to write to
    #[clap(short, long)]
    pub output: Option<PathBuf>,
    /// Method to test against
    /// Possible values: GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, TRACE, PATCH
    #[clap(short, long, default_value = "GET")]
    pub method: String,
    /// The content type to use
    #[clap(short, long, default_value = "text/plain")]
    pub content_type: String,
    /// Optional data to send with the request
    #[clap(short, long)]
    pub data: Option<String>,
    /// Whether or not to crawl case insensitive
    #[clap(short = 'I', long)]
    pub case_insensitive: bool,
}
