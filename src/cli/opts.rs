use std::path::Path;

use crate::utils::constants::{
    DEFAULT_FOLLOW_REDIRECTS, DEFAULT_METHOD, DEFAULT_SAVE_FILE, DEFAULT_TIMEOUT,
};
use field_accessor_pub::FieldAccessor;
use serde::{Deserialize, Serialize};

use super::helpers::{
    parse_cookie, parse_header, parse_method, parse_url, parse_wordlist, KeyOrKeyVal,
    KeyOrKeyValParser, KeyVal, KeyValParser,
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
    #[clap(
        value_parser = parse_url,
        env,
        hide_env=true
    )]
    #[serde(default)]
    pub url: Option<String>,

    /// Wordlist(s)
    #[clap(
        value_name = "FILE:KEY",
        env,
        hide_env = true,
        value_parser = parse_wordlist
    )]
    #[merge(strategy = merge::vec::overwrite_empty)]
    #[serde(default)]
    pub wordlists: Vec<Wordlist>,

    /// Crawl mode
    #[clap(
        short,
        long,
        value_name = "MODE",
        value_parser = clap::builder::PossibleValuesParser::new(["recursive", "recursion", "r", "classic", "c"]),
        env,
        hide_env = true
    )]
    #[serde(default)]
    pub mode: Option<String>,

    /// Force scan even if the target is not responding
    #[clap(long, env, hide_env = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub force: bool,

    /// Consider connection errors as a hit
    #[clap(long, env, hide_env = true, visible_alias = "hce", help_heading = Some("Responses"))]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
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
    #[clap(long, default_value = DEFAULT_TIMEOUT.to_string(), env, hide_env = true, visible_alias = "to", help_heading = Some("Requests"))]
    pub timeout: Option<usize>,

    /// User agent
    #[clap(short, long, env, hide_env = true, help_heading = Some("Requests"))]
    pub user_agent: Option<String>,

    /// HTTP method
    #[clap(short = 'X', long, default_value = DEFAULT_METHOD, value_parser = parse_method, env, hide_env=true, help_heading = Some("Requests"))]
    pub method: Option<String>,

    /// Data to send with the request
    #[clap(short = 'D', long, env, hide_env = true, help_heading = Some("Requests"),)]
    pub data: Option<String>,

    /// Headers to send
    #[clap(short = 'H', long, value_name = "key:value", value_parser = parse_header, env, hide_env=true, help_heading = Some("Requests"),)]
    #[merge(strategy = merge::vec::overwrite_empty)]
    #[serde(default)]
    pub headers: Vec<String>,

    /// Cookies to send
    #[clap(short = 'C', long, value_name = "key=value", value_parser = parse_cookie, env, hide_env=true, help_heading = Some("Requests"),)]
    #[merge(strategy = merge::vec::overwrite_empty)]
    #[serde(default)]
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

    /// Configuration file
    #[clap(short, long, env, hide_env = true)]
    pub config: Option<String>,

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
    #[serde(default)]
    pub no_color: bool,

    /// Quiet mode
    #[clap(short, long, env, hide_env = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub quiet: bool,

    /// Interactive mode
    #[clap(short, long, env, hide_env = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub interactive: bool,

    /// Insecure mode, disables SSL certificate validation
    #[clap(long, env, hide_env = true, visible_alias = "unsecure")]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub insecure: bool,

    /// Show response additional body information
    #[clap(
        long,
        env,
        hide_env = true,
        help_heading = Some("Responses"),
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
                    "cookies",
                    "type"
                ]
            )
        )
    )]
    #[merge(strategy = merge::vec::overwrite_empty)]
    #[serde(default)]
    pub show: Vec<String>,

    /// Resume from a saved file
    #[clap(short='r', long, help_heading = Some("Resume"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub resume: bool,

    /// Custom save file
    #[clap(long, default_value = Some(DEFAULT_SAVE_FILE), help_heading = Some("Resume"), value_name = "FILE", env, hide_env=true)]
    pub save_file: Option<String>,

    /// Don't save the state in case you abort
    #[clap(long, help_heading = Some("Resume"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub no_save: bool,

    /// Keep the save file after finishing when using --resume
    #[clap(long, help_heading = Some("Resume"), env, hide_env=true, visible_alias = "keep")]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub keep_save: bool,

    /// Wordlist transformations: "lower", "upper", "prefix", "suffix", "capitalize", "reverse", "remove", "replace"
    #[clap(short='T', long, help_heading = Some("Wordlists"), env, hide_env=true, value_parser(KeyOrKeyValParser))]
    #[merge(strategy = merge::vec::overwrite_empty)]
    #[serde(default)]
    pub transform: Vec<KeyOrKeyVal<String, String>>,

    /// Wordlist filtering: "contains", "starts", "ends", "regex", "length"
    #[clap(short='w', long, help_heading = Some("Wordlists"), value_name = "KEY:FILTER", env, hide_env=true, value_parser(KeyValParser), visible_alias = "wf")]
    #[merge(strategy = merge::vec::overwrite_empty)]
    #[serde(default)]
    pub wordlist_filter: Vec<KeyVal<String, String>>,

    /// Response filtering: "time", "status", "contains", "starts", "end", "regex", "length", "hash", "header", "json", "depth", "type"
    #[clap(
        short,
        long,
        help_heading = Some("Responses"),
        value_name = "KEY:FILTER",
        env,
        hide_env=true,
        value_parser(KeyValParser)
    )]
    #[merge(strategy = merge::vec::overwrite_empty)]
    #[serde(default)]
    pub filter: Vec<KeyVal<String, String>>,

    /// Treat filters as or instead of and
    #[clap(long, help_heading = Some("Responses"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub or: bool,

    /// Force the recursion over non-directories
    #[clap(long, help_heading = Some("Responses"), env, hide_env=true, visible_alias = "fr")]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub force_recursion: bool,

    /// Request file (.http, .rest)
    #[clap(long, value_name = "FILE", env, hide_env = true, visible_alias = "rf", help_heading = Some("Requests"),)]
    pub request_file: Option<String>,

    /// Proxy URL
    #[clap(short='P', long, help_heading = Some("Proxy"), value_name = "URL", env, hide_env=true)]
    pub proxy: Option<String>,

    /// Proxy username and password
    #[clap(long, help_heading = Some("Proxy"), value_name = "USER:PASS", env, hide_env=true)]
    pub proxy_auth: Option<String>,

    /// Generate markdown help - for developers
    #[clap(long, hide = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub generate_markdown: bool,

    /// Generate shell completions - for developers
    #[clap(long, hide = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub generate_completions: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct Wordlist(pub String, pub Vec<String>);

impl Wordlist {
    pub fn new(file: String, keys: Vec<String>) -> Self {
        Self(file, keys)
    }
}

impl<'de> Deserialize<'de> for Wordlist {
    fn deserialize<D>(deserializer: D) -> Result<Wordlist, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts = s.split(':').collect::<Vec<_>>();
        let file = parts[0].to_string();
        let keys = parts[1..]
            .iter()
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            })
            .collect();
        Ok(Wordlist(file, keys))
    }
}

impl Serialize for Wordlist {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{}:{}", self.0, self.1.join(",")).serialize(serializer)
    }
}

impl Opts {
    pub async fn from_path<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let contents = tokio::fs::read_to_string(path).await?;
        let opts: Opts = toml::from_str(&contents)?;
        Ok(opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_opts_env() {
        env::set_var("URL", "http://example.com");
        env::set_var("WORDLISTS", "wordlist1.txt:w1");
        env::set_var("METHOD", "GET");
        env::set_var("TIMEOUT", "10");
        env::set_var("HEADERS", "key:value");
        env::set_var("COOKIES", "key=value");
        env::set_var("FOLLOW_REDIRECTS", "5");
        env::set_var("THREADS", "10");
        env::set_var("DEPTH", "5");
        env::set_var("OUTPUT", "output.txt");
        env::set_var("USER_AGENT", "user-agent");
        env::set_var("DATA", "data");
        env::set_var("THROTTLE", "100");
        env::set_var("MAX_TIME", "100");
        env::set_var("NO_COLOR", "true");
        env::set_var("QUIET", "true");
        env::set_var("INTERACTIVE", "true");
        env::set_var("INSECURE", "true");
        env::set_var("SHOW", "length");
        env::set_var("RESUME", "true");
        env::set_var("SAVE_FILE", "save_file.txt");
        env::set_var("NO_SAVE", "true");
        env::set_var("KEEP_SAVE", "true");
        env::set_var("TRANSFORM", "lower");
        env::set_var("WORDLIST_FILTER", "length:5");
        env::set_var("FILTER", "length:5");
        env::set_var("OR", "true");
        env::set_var("PROXY", "http://proxy.com");
        env::set_var("PROXY_AUTH", "user:pass");

        let opts = Opts::parse_from(vec![""]);
        assert_eq!(opts.url, Some("http://example.com/".to_string()));
        assert_eq!(
            opts.wordlists,
            vec![Wordlist(
                "wordlist1.txt".to_string(),
                vec!["w1".to_string()]
            )]
        );
        assert_eq!(opts.method, Some("GET".to_string()));
        assert_eq!(opts.timeout, Some(10));
        assert_eq!(opts.headers, vec!["key:value".to_string()]);
        assert_eq!(opts.cookies, vec!["key=value".to_string()]);
        assert_eq!(opts.follow_redirects, Some(5));
        assert_eq!(opts.threads, Some(10));
        assert_eq!(opts.depth, Some(5));
        assert_eq!(opts.output, Some("output.txt".to_string()));
        assert_eq!(opts.user_agent, Some("user-agent".to_string()));
        assert_eq!(opts.data, Some("data".to_string()));
        assert_eq!(opts.throttle, Some(100));
        assert_eq!(opts.max_time, Some(100));
        assert!(opts.no_color);
        assert!(opts.quiet);
        assert!(opts.interactive);
        assert!(opts.insecure);
        assert_eq!(opts.show, vec!["length".to_string()]);
        assert!(opts.resume);
        assert_eq!(opts.save_file, Some("save_file.txt".to_string()));
        assert!(opts.no_save);
        assert!(opts.keep_save);
        assert_eq!(opts.transform, vec![KeyOrKeyVal("lower".to_string(), None)]);
        assert_eq!(
            opts.wordlist_filter,
            vec![KeyVal("length".to_string(), "5".to_string())]
        );
        assert_eq!(
            opts.filter,
            vec![KeyVal("length".to_string(), "5".to_string())]
        );
        assert!(opts.or);
        assert_eq!(opts.proxy, Some("http://proxy.com".to_string()));
        assert_eq!(opts.proxy_auth, Some("user:pass".to_string()));
    }
}
