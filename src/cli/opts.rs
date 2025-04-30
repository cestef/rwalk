use std::path::Path;

use crate::utils::{
    constants::{
        DEFAULT_DEPTH, DEFAULT_FOLLOW_REDIRECTS, DEFAULT_METHOD, DEFAULT_MODE, DEFAULT_SAVE_FILE,
        DEFAULT_TIMEOUT,
    },
    version,
};
use serde::{Deserialize, Serialize};

use super::helpers::{
    parse_cookie, parse_header, parse_host, parse_method, parse_url, parse_wordlist, KeyOrKeyVal,
    KeyOrKeyValParser, KeyVal, KeyValParser,
};
use clap::Parser;
use color_eyre::eyre::Result;
use merge::Merge;

#[derive(Parser, Clone, Debug, Default, Serialize, Deserialize, Merge)]
#[clap(
    version = version(),
    author = "cstef",
    about = "A blazingly fast web directory scanner"
)]
pub struct Opts {
    /// Target URL
    #[clap(
        value_parser = parse_url,
        env,
        hide_env=true
    )]
    #[merge(strategy = overwrite_option)]
    #[serde(default)]
    pub url: Option<String>,

    /// Wordlist(s)
    #[clap(
        value_name = "FILE:KEY",
        env,
        hide_env = true,
        value_parser = parse_wordlist,
    )]
    #[merge(strategy = overwrite_vec)]
    #[serde(default)]
    pub wordlists: Vec<Wordlist>,

    /// Crawl mode
    #[clap(
        short,
        long,
        value_name = "MODE",
        env,
        hide_env = true,
        help_heading = Some("Mode")
    )]
    #[merge(strategy = overwrite_option)]
    #[serde(default = "default_mode")]
    pub mode: Option<String>,

    /// Depth to crawl
    #[clap(
        short,
        long,
        value_name = "DEPTH",
        env,
        hide_env = true,
        help_heading = Some("Mode")
    )]
    #[merge(strategy = overwrite_option)]
    #[serde(default = "default_depth")]
    pub depth: Option<usize>,

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
    #[merge(strategy = overwrite_option)]
    pub threads: Option<usize>,

    /// Follow redirects
    #[clap(short = 'R', long, value_name = "COUNT", env, hide_env = true)]
    #[merge(strategy = overwrite_option)]
    #[serde(default = "default_follow_redirects")]
    pub follow_redirects: Option<usize>,

    /// Output file
    #[clap(short, long, value_name = "FILE", env, hide_env = true)]
    #[merge(strategy = overwrite_option)]
    pub output: Option<String>,

    /// Pretty format the output (only JSON)
    #[clap(long, env, hide_env = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub pretty: bool,

    /// Request timeout in seconds
    #[clap(long, env, hide_env = true, visible_alias = "to", help_heading = Some("Requests"))]
    #[merge(strategy = overwrite_option)]
    #[serde(default = "default_timeout")]
    pub timeout: Option<usize>,

    /// User agent
    #[clap(short, long, env, hide_env = true, help_heading = Some("Requests"))]
    #[merge(strategy = overwrite_option)]
    pub user_agent: Option<String>,

    /// HTTP method
    #[clap(short = 'X', long, value_parser = parse_method, env, hide_env=true, help_heading = Some("Requests"))]
    #[merge(strategy = overwrite_option)]
    #[serde(default = "default_method")]
    pub method: Option<String>,

    /// Data to send with the request
    #[clap(short = 'D', long, env, hide_env = true, help_heading = Some("Requests"),)]
    #[merge(strategy = overwrite_option)]
    pub data: Option<String>,

    /// Headers to send
    #[clap(short = 'H', long, value_name = "key:value", value_parser = parse_header, env, hide_env=true, help_heading = Some("Requests"),value_delimiter = ',')]
    #[merge(strategy = overwrite_vec)]
    #[serde(default)]
    pub headers: Vec<String>,

    /// Cookies to send
    #[clap(short = 'C', long, value_name = "key=value", value_parser = parse_cookie, env, hide_env=true, help_heading = Some("Requests"),value_delimiter = ',')]
    #[merge(strategy = overwrite_vec)]
    #[serde(default)]
    pub cookies: Vec<String>,

    /// Save file
    #[clap(short = 's', long, env, hide_env = true, help_heading = Some("Save"))]
    #[merge(strategy = overwrite_option)]
    #[serde(default = "default_save_file")]
    pub save_file: Option<String>,

    /// Don't save results
    #[clap(short = 'n', long, env, hide_env = true, help_heading = Some("Save"))]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub no_save: bool,

    /// Keep save file after completion
    #[clap(long, env, hide_env = true, help_heading = Some("Save"))]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub keep_save: bool,

    /// Resume from a saved file
    #[clap(short='r', long, help_heading = Some("Resume"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub resume: bool,

    /// Configuration file
    #[clap(short, long, env, hide_env = true)]
    #[merge(strategy = overwrite_option)]
    pub config: Option<String>,

    /// Request throttling (requests per second) per thread
    #[clap(long, env, hide_env = true)]
    #[merge(strategy = overwrite_option)]
    pub throttle: Option<usize>,

    /// Max time to run (will abort after given time) in seconds
    #[clap(short = 'M', long, env, hide_env = true)]
    #[merge(strategy = overwrite_option)]
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

    /// Distribute the requests to multiple hosts
    #[clap(
        long,
        env,
        hide_env = true,
        value_delimiter = ',',
        visible_alias = "distribute",
        value_parser = parse_host
    )]
    #[merge(strategy = overwrite_vec)]
    #[serde(default)]
    pub distributed: Vec<String>,

    /// Show response additional body information
    #[clap(
        long,
        env,
        hide_env = true,
        help_heading = Some("Responses"),
        value_delimiter = ','
    )]
    #[merge(strategy = overwrite_vec)]
    #[serde(default)]
    pub show: Vec<String>,

    /// Wordlist transformations: "lower", "upper", "prefix", "suffix", "capitalize", "reverse", "remove", "replace", "encode"
    #[clap(short='T', long, help_heading = Some("Wordlists"), env, hide_env=true, value_parser(KeyOrKeyValParser), value_delimiter = ',')]
    #[merge(strategy = overwrite_vec)]
    #[serde(default)]
    pub transform: Vec<KeyOrKeyVal<String, String>>,

    /// Wordlist filtering: "contains", "starts", "ends", "regex", "length"
    #[clap(short='w', long, help_heading = Some("Wordlists"), value_name = "KEY:FILTER", env, hide_env=true, value_parser(KeyValParser), visible_alias = "wf", value_delimiter = ',')]
    #[merge(strategy = overwrite_vec)]
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
        value_parser(KeyValParser),
        value_delimiter = ';'
    )]
    #[merge(strategy = overwrite_vec)]
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

    /// Override the default directory detection method with your own rhai script
    #[clap(long, help_heading = Some("Responses"), env, hide_env=true, visible_alias = "ds", visible_alias = "dir-script")]
    #[merge(strategy = overwrite_option)]
    pub directory_script: Option<String>,

    /// Request file (.http, .rest)
    #[clap(long, value_name = "FILE", env, hide_env = true, visible_alias = "rf", help_heading = Some("Requests"),)]
    #[merge(strategy = overwrite_option)]
    pub request_file: Option<String>,

    /// Proxy URL
    #[clap(short='P', long, help_heading = Some("Proxy"), value_name = "URL", env, hide_env=true)]
    #[merge(strategy = overwrite_option)]
    pub proxy: Option<String>,

    /// Proxy username and password
    #[clap(long, help_heading = Some("Proxy"), value_name = "USER:PASS", env, hide_env=true)]
    #[merge(strategy = overwrite_option)]
    pub proxy_auth: Option<String>,

    /// Allow subdomains to be scanned in spider mode
    #[clap(long, help_heading = Some("Spider"), env, hide_env=true, visible_alias = "sub")]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub subdomains: bool,

    /// Allow external domains to be scanned in spider mode (Warning: this can generate a lot of traffic)
    #[clap(long, help_heading = Some("Spider"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub external: bool,

    /// Attributes to be crawled in spider mode
    #[clap(short, long, help_heading = Some("Spider"), env, hide_env=true, value_delimiter = ',')]
    #[merge(strategy = overwrite_vec)]
    #[serde(default)]
    pub attributes: Vec<String>,

    /// Scripts to run after each request
    #[clap(long, help_heading = Some("Scripts"), env, hide_env=true, visible_alias = "sc", value_delimiter = ',')]
    #[merge(strategy = overwrite_vec)]
    #[serde(default)]
    pub scripts: Vec<String>,

    /// Ignore scripts errors
    #[clap(long, help_heading = Some("Scripts"), env, hide_env=true, visible_alias = "ise")]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub ignore_scripts_errors: bool,

    /// Generate markdown help - for developers
    #[clap(long, hide = true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub generate_markdown: bool,

    /// Generate completions for the specified shell
    #[clap(long, value_name = "SHELL", env, hide_env = true)]
    #[merge(strategy = overwrite_option)]
    #[serde(default)]
    pub completions: Option<String>,

    /// Open the config in the default editor (EDITOR)
    #[clap(long, help_heading = Some("Debug"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub open_config: bool,

    /// Print the default config in TOML format. Useful for debugging and creating your own config
    #[clap(long, help_heading = Some("Debug"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub default_config: bool,

    /// Capture the responses to be analyzed later in the interactive mode
    #[clap(long, help_heading = Some("Interactive"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub capture: bool,

    /// Skip all confirmation prompts
    #[clap(short, long, help_heading = Some("Interactive"), env, hide_env=true)]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    pub yes: bool,

    /// Random wait time range in seconds between requests, e.g. 0.5-1.5
    #[clap(long, help_heading = Some("Requests"), env, hide_env=true)]
    #[merge(strategy = overwrite_option)]
    pub wait: Option<String>,
}

// Updates with the latest value, replacing the current one if provided.
fn overwrite_option<T>(a: &mut Option<T>, b: Option<T>) {
    if b.is_some() {
        *a = b;
    }
}

// Updates with the latest value, replacing the current one if provided.
fn overwrite_vec<T>(a: &mut Vec<T>, b: Vec<T>) {
    if !b.is_empty() {
        *a = b;
    }
}

fn default_mode() -> Option<String> {
    Some(DEFAULT_MODE.to_string())
}

fn default_depth() -> Option<usize> {
    Some(DEFAULT_DEPTH)
}

fn default_follow_redirects() -> Option<usize> {
    Some(DEFAULT_FOLLOW_REDIRECTS)
}

fn default_timeout() -> Option<usize> {
    Some(DEFAULT_TIMEOUT)
}

fn default_method() -> Option<String> {
    Some(DEFAULT_METHOD.to_string())
}

fn default_save_file() -> Option<String> {
    Some(DEFAULT_SAVE_FILE.to_string())
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
        format!(
            "{}{}",
            self.0,
            if !self.1.is_empty() {
                format!(":{}", self.1.join(","))
            } else {
                "".to_string()
            }
        )
        .serialize(serializer)
    }
}

// deserialize keyorkeyval

impl<'de> Deserialize<'de> for KeyOrKeyVal<String, String> {
    fn deserialize<D>(deserializer: D) -> Result<KeyOrKeyVal<String, String>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts = s.split(':').collect::<Vec<_>>();
        if parts.len() == 1 {
            Ok(KeyOrKeyVal(parts[0].to_string(), None))
        } else {
            Ok(KeyOrKeyVal(
                parts[0].to_string(),
                Some(parts[1].to_string()),
            ))
        }
    }
}

impl Serialize for KeyOrKeyVal<String, String> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!(
            "{}{}",
            self.0,
            if let Some(v) = &self.1 {
                format!(":{}", v)
            } else {
                "".to_string()
            }
        )
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for KeyVal<String, String> {
    fn deserialize<D>(deserializer: D) -> Result<KeyVal<String, String>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts = s.split(':').collect::<Vec<_>>();
        Ok(KeyVal(parts[0].to_string(), parts[1].to_string()))
    }
}

impl Serialize for KeyVal<String, String> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{}:{}", self.0, self.1).serialize(serializer)
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
