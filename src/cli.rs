use clap::Parser;
use lazy_static::lazy_static;
use url::Url;

use crate::{constants::SAVE_FILE, utils::parse_range_input};
#[derive(Parser, Clone, Debug)]
#[clap(
    version,
    author = "cstef",
    about = "A blazing fast web directory scanner"
)]
pub struct Opts {
    /// Target URL
    #[clap(required = true, value_parser = parse_url)]
    pub url: String,
    /// Wordlist(s)
    #[clap(required = true)]
    pub wordlists: Vec<String>,
    /// Number of threads to use
    #[clap(short, long)]
    pub threads: Option<usize>,
    /// Maximum depth to crawl
    #[clap(short, long, default_value = "1")]
    pub depth: usize,
    /// Output file
    #[clap(short, long, value_name = "FILE")]
    pub output: Option<String>,
    /// Request timeout in seconds
    #[clap(short = 'T', long, default_value = "10")]
    pub timeout: u64,
    /// User agent
    #[clap(short, long)]
    pub user_agent: Option<String>,
    /// Quiet mode
    #[clap(short, long)]
    pub quiet: bool,
    /// HTTP method
    #[clap(short, long, default_value = "GET", value_parser = method_exists)]
    pub method: String,
    /// Data to send with the request
    #[clap(short, long)]
    pub data: Option<String>,
    /// Headers to send
    #[clap(short = 'H', long, value_name = "key:value", value_parser = is_header)]
    pub headers: Vec<String>,
    /// Cookies to send
    #[clap(short, long, value_name = "key=value", value_parser = is_cookie)]
    pub cookies: Vec<String>,
    /// Follow redirects
    #[clap(short = 'R', long, default_value = "0", value_name = "COUNT")]
    pub follow_redirects: usize,
    /// Request throttling (requests per second) per thread
    #[clap(long, default_value = "0")]
    pub throttle: usize,

    /// Resume from a saved file
    #[clap(long, help_heading = Some("Resume"))]
    pub resume: bool,
    /// Custom save file
    #[clap(short = 'f', long, default_value = SAVE_FILE, help_heading = Some("Resume"), value_name = "FILE")]
    pub save_file: String,
    /// Don't save the state in case you abort
    #[clap(long, help_heading = Some("Resume"))]
    pub no_save: bool,

    /// Wordlist to uppercase
    #[clap(short='L', long, help_heading = Some("Transformations"), conflicts_with = "transform_upper")]
    pub transform_lower: bool,
    /// Wordlist to lowercase
    #[clap(short='U', long, help_heading = Some("Transformations"), conflicts_with = "transform_lower")]
    pub transform_upper: bool,
    /// Append a prefix to each word
    #[clap(short='P', long, help_heading = Some("Transformations"), value_name = "PREFIX")]
    pub transform_prefix: Option<String>,
    /// Append a suffix to each word
    #[clap(short='S', long, help_heading = Some("Transformations"), value_name = "SUFFIX")]
    pub transform_suffix: Option<String>,
    /// Capitalize each word
    #[clap(short='C', long, help_heading = Some("Transformations"), conflicts_with_all = &["transform_lower", "transform_upper"])]
    pub transform_capitalize: bool,

    /// Contains the specified string
    #[clap(long, help_heading = Some("Wordlist Filtering"), value_name = "STRING", visible_alias = "wfc")]
    pub wordlist_filter_contains: Option<String>,
    /// Start with the specified string
    #[clap(long, help_heading = Some("Wordlist Filtering"), value_name = "STRING", visible_alias = "wfs")]
    pub wordlist_filter_starts_with: Option<String>,
    /// End with the specified string
    #[clap(long, help_heading = Some("Wordlist Filtering"), value_name = "STRING", visible_alias = "wfe")]
    pub wordlist_filter_ends_with: Option<String>,
    /// Match the specified regex
    #[clap(long, help_heading = Some("Wordlist Filtering"), value_name = "REGEX", visible_alias = "wfr")]
    pub wordlist_filter_regex: Option<String>,
    /// Length range
    /// e.g.: 5, 5-10, 5,10,15, >5, <5
    #[clap(long, help_heading = Some("Wordlist Filtering"), value_name = "RANGE", visible_alias = "wfl", value_parser(parse_cli_range_input))]
    pub wordlist_filter_length: Option<String>,

    /// Reponse status code,
    /// e.g.: 200, 200-300, 200,300,400, >200, <200
    #[clap(long, help_heading = Some("Response Filtering"), value_name = "RANGE", visible_alias = "fsc", value_parser(parse_cli_range_input))]
    pub filter_status_code: Option<String>,
    /// Contains the specified string
    #[clap(long, help_heading = Some("Response Filtering"), value_name = "STRING", visible_alias = "fc")]
    pub filter_contains: Option<String>,
    /// Start with the specified string
    #[clap(long, help_heading = Some("Response Filtering"), value_name = "STRING", visible_alias = "fs")]
    pub filter_starts_with: Option<String>,
    /// End with the specified string
    #[clap(long, help_heading = Some("Response Filtering"), value_name = "STRING", visible_alias = "fe")]
    pub filter_ends_with: Option<String>,
    /// Match the specified regex
    #[clap(long, help_heading = Some("Response Filtering"), value_name = "REGEX", visible_alias = "fr")]
    pub filter_regex: Option<String>,
    /// Response length
    /// e.g.: 100, >100, <100, 100-200, 100,200,300
    #[clap(long, help_heading = Some("Response Filtering"), value_name = "RANGE", visible_alias = "fl", value_parser(parse_cli_range_input))]
    pub filter_length: Option<String>,
    /// Response time range in milliseconds
    /// e.g.: >1000, <1000, 1000-2000
    #[clap(long, help_heading = Some("Response Filtering"), value_name = "RANGE", visible_alias = "ft", value_parser(parse_cli_range_input))]
    pub filter_time: Option<String>,
}

fn parse_cli_range_input(s: &str) -> Result<String, String> {
    parse_range_input(s)?;
    Ok(s.to_string())
}

fn parse_url(s: &str) -> Result<String, String> {
    let s = if !s.starts_with("http://") && !s.starts_with("https://") {
        format!("http://{}", s)
    } else {
        s.to_string()
    };
    let url = Url::parse(&s);
    match url {
        Ok(url) => Ok(url.to_string()),
        Err(_) => Err("Invalid URL".to_string()),
    }
}

fn is_header(s: &str) -> Result<String, String> {
    // key: value
    let parts = s.split(":").collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err("Invalid header".to_string());
    }
    Ok(s.to_string())
}

fn is_cookie(s: &str) -> Result<String, String> {
    // key=value
    let parts = s.split("=").collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err("Invalid cookie".to_string());
    }
    Ok(s.to_string())
}

fn method_exists(s: &str) -> Result<String, String> {
    let methods = vec![
        "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "TRACE", "CONNECT",
    ];
    let s = s.to_uppercase();
    if methods.contains(&s.as_str()) {
        Ok(s.to_string())
    } else {
        Err("Invalid HTTP method".to_string())
    }
}

lazy_static! {
    #[derive(Debug)]
    pub static ref OPTS: Opts = Opts::parse();
}
