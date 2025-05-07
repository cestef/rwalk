use std::path::PathBuf;

use crate::{
    constants::THREADS_PER_CORE,
    types::EngineMode,
    utils::types::{HTTPMethod, IntRange, ListType, ThrottleMode},
};
use clap::Parser;
use clap::builder::EnumValueParser;
use dashmap::DashSet as HashSet;
use dyn_fields::DynamicFields;
use merge::Merge;
use parse::{
    parse_filter, parse_keyed_key_or_keyval, parse_keyed_keyval, parse_throttle, parse_url,
    parse_wordlist,
};
use serde::{Deserialize, Serialize};
use url::Url;

pub mod help;
pub mod interactive;
pub mod parse;
pub mod serialize;
pub mod utils;

const SUBCOMMANDS_FLAGS: [&str; 5] = [
    "help",
    "help_long",
    "list",
    "generate_markdown",
    "interactive",
];

#[derive(Debug, Parser, Clone, Merge, Deserialize, Serialize, DynamicFields)]
#[clap(version = utils::version(), long_version = utils::long_version(), disable_help_flag = true, help_template = "{all-args}")]
pub struct Opts {
    //
    // ------------------------------------------------------------------------
    // Internal flags
    // ------------------------------------------------------------------------
    //
    #[clap(short, hide = true)]
    #[merge(skip)]
    #[dyn_fields(skip = true)]
    pub help: bool,

    #[clap(long = "help", hide = true)]
    #[merge(skip)]
    #[dyn_fields(skip = true)]
    pub help_long: bool,

    /// Generate markdown help
    #[merge(skip)]
    #[clap(long, hide = true)]
    #[dyn_fields(skip = true)]
    pub generate_markdown: bool,

    //
    // ------------------------------------------------------------------------
    // Core options
    // ------------------------------------------------------------------------
    //
    /// URL to scan
    #[clap(value_parser = parse_url, required_unless_present_any(SUBCOMMANDS_FLAGS))]
    #[merge(strategy = merge_overwrite)]
    #[dyn_fields(set = "transformers::url::set")]
    pub url: Option<Url>,

    /// Wordlist file(s) to use, `path[:key]`
    #[clap(value_parser = parse_wordlist, required_unless_present_any(SUBCOMMANDS_FLAGS))]
    #[merge(strategy = merge::vec::append)]
    #[dyn_fields(
        alias = "wordlist",
        set = "transformers::wordlist::set",
        get = "transformers::wordlist::get"
    )]
    #[serde(
        serialize_with = "serialize::wordlist::ser",
        deserialize_with = "serialize::wordlist::de"
    )]
    pub wordlists: Vec<(String, String)>,

    /// Number of threads to use, defaults to `num_cores * 5`
    #[clap(short = 'T', long, default_value_t = num_cpus::get() * THREADS_PER_CORE, help_heading = "Core")]
    #[merge(strategy = merge_overwrite)]
    pub threads: usize,

    /// Request rate limit in requests per second
    #[clap(long, visible_alias = "rate", help_heading = "Core", value_parser = parse_throttle)]
    #[merge(strategy = merge::option::overwrite_none)]
    pub throttle: Option<(u64, ThrottleMode)>,

    /// Fuzzing mode
    #[clap(short, long, default_value = "recursive", value_parser = EnumValueParser::<EngineMode>::new(), help_heading = "Core")]
    #[merge(strategy = merge_overwrite)]
    pub mode: EngineMode,

    //
    // ------------------------------------------------------------------------
    // Scan Configuration
    // ------------------------------------------------------------------------
    //
    /// Maximum depth in recursive mode
    #[clap(short, long, default_value = "0", help_heading = "Scan Configuration")]
    #[merge(strategy = merge_overwrite)]
    pub depth: usize,

    /// Maximum retries for failed requests
    #[clap(
        short,
        long,
        default_value = "3",
        visible_alias = "retry",
        help_heading = "Scan Configuration"
    )]
    #[merge(strategy = merge_overwrite)]
    pub retries: usize,

    /// What status codes to retry on
    #[clap(long, visible_alias = "retry-on", help_heading = "Scan Configuration")]
    #[merge(strategy = merge_overwrite)]
    pub retry_codes: Vec<IntRange<u16>>,

    /// Force the scan, even if the target is unreachable
    #[clap(long, help_heading = "Scan Configuration")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub force: bool,

    /// Force the recursion, even if the URL is not detected as a directory
    #[clap(long, visible_alias = "fr", help_heading = "Scan Configuration")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub force_recursion: bool,

    //
    // ------------------------------------------------------------------------
    // Filters and Transforms
    // ------------------------------------------------------------------------
    //
    /// List of filters to apply to responses, see `--list-filters`
    #[clap(short, long, value_parser = parse_filter, visible_alias = "filter", value_name ="EXPR", help_heading = "Scan Configuration")]
    #[merge(strategy = merge::vec::append)]
    pub filters: Vec<String>,

    /// List of transformations to apply to wordlists, see `--list-transforms`
    #[clap(short, long, value_parser = parse_keyed_key_or_keyval, value_delimiter = ';', visible_alias = "transform", value_name = "TRANSFORM", help_heading = "Scan Configuration")]
    #[merge(strategy = merge::vec::append)]
    pub transforms: Vec<(HashSet<String>, String, Option<String>)>,

    /// Wordlist filters, see `--list-filters`
    #[clap(
        short,
        long,
        visible_alias = "wf",
        value_name = "EXPR",
        help_heading = "Scan Configuration"
    )]
    #[merge(strategy = merge::option::overwrite_none)]
    pub wordlist_filter: Option<String>,

    //
    // ------------------------------------------------------------------------
    // Request Control
    // ------------------------------------------------------------------------
    //
    /// HTTP method to use
    #[clap(short = 'X', long, value_parser = EnumValueParser::<HTTPMethod>::new(), default_value = "GET", help_heading = "Request Control")]
    #[merge(strategy = merge_overwrite)]
    pub method: HTTPMethod,

    /// Headers to send with the request, `name:value`
    #[clap(short = 'H', long, value_delimiter = ';', value_name = "HEADER", value_parser = parse_keyed_keyval, help_heading = "Request Control")]
    #[merge(strategy = merge::vec::append)]
    pub headers: Vec<(HashSet<String>, String, String)>,

    /// Only use HTTP/1
    #[clap(long, help_heading = "Request Control")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub http1: bool,

    /// Only use HTTP/2
    #[clap(long, help_heading = "Request Control")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub http2: bool,

    //
    // ------------------------------------------------------------------------
    // Output
    // ------------------------------------------------------------------------
    //
    /// Extra information to display on hits
    #[clap(short, long, value_delimiter = ',', help_heading = "Output")]
    #[merge(strategy = merge::vec::append)]
    pub show: Vec<String>,

    /// Save responses to a file, supported: `json`, `csv`, `txt`, `md`
    #[clap(short, long, help_heading = "Output")]
    #[merge(strategy = merge::option::overwrite_none)]
    pub output: Option<PathBuf>,

    /// Ring the terminal bell on hits
    #[clap(
        long,
        visible_alias = "ding",
        visible_alias = "dong",
        help_heading = "Output"
    )]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub bell: bool,

    /// List both available filters and wordlist transforms
    #[merge(skip)]
    #[clap(long, help_heading = "Output", value_parser = EnumValueParser::<ListType>::new())]
    pub list: Option<ListType>,

    //
    // ------------------------------------------------------------------------
    // Advanced
    // ------------------------------------------------------------------------
    //
    /// Resume from previous session
    #[clap(long, help_heading = "Advanced")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub resume: bool,

    /// Don't save state on `Ctrl+C`
    #[clap(long, help_heading = "Advanced")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub no_save: bool,

    /// Load configuration from a file, merges with command line arguments
    #[merge(skip)]
    #[clap(short, long, help_heading = "Advanced")]
    pub config: Option<PathBuf>,

    /// Interactive mode
    #[clap(short, long, help_heading = "Advanced")]
    #[merge(strategy = merge::bool::overwrite_false)]
    #[dyn_fields(skip = true)]
    pub interactive: bool,

    /// Include comment lines from the wordlists
    #[clap(long, help_heading = "Advanced")]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub include_comments: bool,
}

fn merge_overwrite<T>(a: &mut T, b: T) {
    *a = b;
}

mod transformers {
    use crate::utils::constants::DEFAULT_WORDLIST_KEY;
    use serde_json::Value;
    pub mod wordlist {
        use super::*;
        /// Parses a wordlist string into a Value::Array with path and key pairs
        pub fn set(value: Value) -> Value {
            match value {
                Value::String(s) => parse_wordlist_string(&s),
                Value::Array(arr) => process_array(arr),
                _ => value,
            }
        }

        pub fn get(value: Value) -> Value {
            match value {
                Value::Array(arr) => {
                    let mut result = Vec::new();
                    for item in arr {
                        if let Value::Array(pair) = item {
                            if pair.len() == 2 {
                                let path = pair[0].as_str().unwrap_or_default();
                                let key = pair[1].as_str().unwrap_or(DEFAULT_WORDLIST_KEY);
                                result.push(Value::String(format!("{}:{}", path, key)));
                            }
                        }
                    }
                    Value::Array(result)
                }
                _ => value,
            }
        }

        /// Processes an array of values, parsing any string values
        fn process_array(arr: Vec<Value>) -> Value {
            let mut result = Vec::new();

            for item in arr {
                match item {
                    Value::String(s) => {
                        let parsed_items = parse_wordlist_string(&s);
                        if let Value::Array(items) = parsed_items {
                            result.extend(items);
                        }
                    }
                    _ => result.push(item),
                }
            }

            Value::Array(result)
        }

        /// Parses a comma-separated list of wordlists into an array of [path, key] pairs
        fn parse_wordlist_string(s: &str) -> Value {
            let wordlists = s.split(',').map(|s| s.trim());
            let result: Vec<Value> = wordlists
                .map(|wordlist| parse_single_wordlist(wordlist))
                .collect();

            Value::Array(result)
        }

        /// Parses a single wordlist entry (either "path:key" or just "path")
        fn parse_single_wordlist(wordlist: &str) -> Value {
            let parts: Vec<&str> = wordlist.split(':').collect();

            let (path, key) = if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                (wordlist, DEFAULT_WORDLIST_KEY)
            };

            Value::Array(vec![
                Value::String(path.to_string()),
                Value::String(key.to_string()),
            ])
        }
    }

    pub mod url {
        use super::*;
        /// Parses a URL string into a Value::String with http:// or https:// prefix
        pub fn set(value: Value) -> Value {
            match value {
                Value::String(ref s) => {
                    // add http:// if not present
                    if !s.starts_with("http://") && !s.starts_with("https://") {
                        Value::String(format!("http://{}", s))
                    } else {
                        value
                    }
                }
                _ => value,
            }
        }
    }
}
