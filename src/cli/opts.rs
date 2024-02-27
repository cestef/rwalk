use crate::utils::constants::{
    DEFAULT_FOLLOW_REDIRECTS, DEFAULT_METHOD, DEFAULT_SAVE_FILE, DEFAULT_TIMEOUT,
};
use field_accessor_pub::FieldAccessor;
use serde::{Deserialize, Serialize};

use super::helpers::{
    parse_cookie, parse_header, parse_key_or_key_val, parse_key_val, parse_method, parse_url,
    parse_wordlist,
};
use anyhow::Result;
use clap::Parser;
use merge::Merge;

#[derive(Parser, Clone, Debug, Default, FieldAccessor, Serialize, Deserialize, Merge)]
#[clap(
    version,
    author = "cstef",
    about = "A blazing fast web directory scanner"
)]
pub struct Opts {
    /// Target URL
    #[clap(required_unless_present = "interactive", required_unless_present = "resume", required_unless_present = "generate_markdown", value_parser = parse_url, env, hide_env=true)]
    pub url: Option<String>,

    /// Wordlist(s)
    #[clap(
        required_unless_present = "interactive",
        required_unless_present = "resume",
        required_unless_present = "generate_markdown",
        env,
        hide_env = true,
        value_parser = parse_wordlist
    )]
    #[merge(strategy = merge::vec::overwrite_empty)]
    pub wordlists: Vec<(String, Vec<String>)>,

    /// Crawl mode
    #[clap(
        short,
        long,
        value_name = "MODE",
        value_parser = clap::builder::PossibleValuesParser::new(["recursive", "recursion", "r", "classic", "c"]),
        env,
        hide_env = true
    )]
    pub mode: Option<String>,

    /// Force scan even if the target is not responding
    #[clap(long, env, hide_env = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub force: bool,

    /// Consider connection errors as a hit
    #[clap(long, env, hide_env = true, visible_alias = "hce")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub hit_connection_errors: bool,

    /// Number of threads to use
    #[clap(short, long, env, hide_env = true)]
    pub threads: Option<usize>,

    /// Crawl recursively until given depth
    #[clap(short, long, env, hide_env = true)]
    pub depth: Option<usize>,

    /// Output file
    #[clap(short, long, value_name = "FILE", env, hide_env = true)]
    pub output: Option<String>,

    /// Request timeout in seconds
    #[clap(long, default_value = DEFAULT_TIMEOUT.to_string(), env, hide_env = true)]
    pub timeout: Option<usize>,

    /// User agent
    #[clap(short, long, env, hide_env = true)]
    pub user_agent: Option<String>,

    /// HTTP method
    #[clap(short = 'X', long, default_value = DEFAULT_METHOD, value_parser = parse_method, env, hide_env=true)]
    pub method: Option<String>,

    /// Data to send with the request
    #[clap(short = 'D', long, env, hide_env = true)]
    pub data: Option<String>,

    /// Headers to send
    #[clap(short = 'H', long, value_name = "key:value", value_parser = parse_header, env, hide_env=true)]
    #[merge(strategy = merge::vec::overwrite_empty)]
    pub headers: Vec<String>,

    /// Cookies to send
    #[clap(short, long, value_name = "key=value", value_parser = parse_cookie, env, hide_env=true)]
    #[merge(strategy = merge::vec::overwrite_empty)]
    pub cookies: Vec<String>,

    /// Follow redirects
    #[clap(
        short = 'R',
        long,
        default_value = DEFAULT_FOLLOW_REDIRECTS.to_string(),
        value_name = "COUNT",
        env,
        hide_env = true
    )]
    pub follow_redirects: Option<usize>,

    /// Request throttling (requests per second) per thread
    #[clap(long, env, hide_env = true)]
    pub throttle: Option<usize>,

    /// Max time to run (will abort after given time) in seconds
    #[clap(short = 'M', long, env, hide_env = true)]
    pub max_time: Option<usize>,

    /// Don't use colors
    /// You can also set the NO_COLOR environment variable
    #[clap(long, alias = "no-colors", env, hide_env = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub no_color: bool,

    /// Quiet mode
    #[clap(short, long, env, hide_env = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub quiet: bool,

    /// Interactive mode
    #[clap(short, long, env, hide_env = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub interactive: bool,

    /// Insecure mode, disables SSL certificate validation
    #[clap(long, env, hide_env = true, visible_alias = "unsecure")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub insecure: bool,

    /// Show response additional body information
    #[clap(
        long,
        env,
        hide_env = true,
        value_parser(
            clap::builder::PossibleValuesParser::new(
                [
                    "length", 
                    "size", 
                    "hash", 
                    "md5", 
                    "headers_length", 
                    "headers_hash", 
                    "body", 
                    "content", 
                    "text", 
                    "headers", 
                    "cookie", 
                    "cookies"
                ]
            )
        )
    )]
    #[merge(strategy = merge::vec::overwrite_empty)]
    pub show: Vec<String>,

    /// Resume from a saved file
    #[clap(short='r', long, help_heading = Some("Resume"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub resume: bool,

    /// Custom save file
    #[clap(long, default_value = Some(DEFAULT_SAVE_FILE), help_heading = Some("Resume"), value_name = "FILE", env, hide_env=true)]
    pub save_file: Option<String>,

    /// Don't save the state in case you abort
    #[clap(long, help_heading = Some("Resume"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub no_save: bool,

    /// Keep the save file after finishing when using --resume
    #[clap(long, help_heading = Some("Resume"), env, hide_env=true, visible_alias = "keep")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub keep_save: bool,

    /// Wordlist transformations: "lower", "upper", "prefix", "suffix", "capitalize", "reverse", "remove", "replace"
    #[clap(short='T', long, help_heading = Some("Transformations"), env, hide_env=true, value_parser(parse_key_or_key_val::<String, String>))]
    #[merge(strategy = merge::vec::overwrite_empty)]
    pub transform: Vec<(String, Option<String>)>,

    /// Wordlist filtering: "contains", "starts", "ends", "regex", "length"
    #[clap(short='w', long, help_heading = Some("Filtering"), value_name = "KEY:FILTER", env, hide_env=true, value_parser(parse_key_val::<String, String>), visible_alias = "wf")]
    #[merge(strategy = merge::vec::overwrite_empty)]
    pub wordlist_filter: Vec<(String, String)>,

    /// Response filtering: "time", "status", "contains", "starts", "end", "regex", "length", "hash", "header", "json", "depth"
    #[clap(short, long, help_heading = Some("Filtering"), value_name = "KEY:FILTER", env, hide_env=true, value_parser(parse_key_val::<String, String>))]
    #[merge(strategy = merge::vec::overwrite_empty)]
    pub filter: Vec<(String, String)>,

    /// Treat filters as or instead of and
    #[clap(long, help_heading = Some("Filtering"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub or: bool,

    /// Proxy URL
    #[clap(short='P', long, help_heading = Some("Proxy"), value_name = "URL", env, hide_env=true)]
    pub proxy: Option<String>,

    /// Proxy username and password
    #[clap(long, help_heading = Some("Proxy"), value_name = "USER:PASS", env, hide_env=true)]
    pub proxy_auth: Option<String>,

    /// Generate markdown help - for developers
    #[clap(long, hide = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub generate_markdown: bool,
}
