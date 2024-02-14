use std::error::Error;

use crate::{constants::SAVE_FILE, utils::parse_range_input};
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use field_accessor::FieldAccessor;
use lazy_static::lazy_static;
use rustyline::DefaultEditor;
use url::Url;

#[derive(Parser, Clone, Debug, Default, FieldAccessor)]
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
        hide_env = true
    )]
    pub wordlists: Vec<String>,

    /// Number of threads to use
    #[clap(short, long, env, hide_env = true)]
    pub threads: Option<usize>,
    /// Maximum depth to crawl
    #[clap(short, long, default_value = "1", env, hide_env = true)]
    pub depth: Option<usize>,
    /// Output file
    #[clap(short, long, value_name = "FILE", env, hide_env = true)]
    pub output: Option<String>,
    /// Request timeout in seconds
    #[clap(long, default_value = "10", env, hide_env = true)]
    pub timeout: Option<usize>,
    /// User agent
    #[clap(short, long, env, hide_env = true)]
    pub user_agent: Option<String>,
    /// HTTP method
    #[clap(short, long, default_value = "GET", value_parser = method_exists, env, hide_env=true)]
    pub method: Option<String>,
    /// Data to send with the request
    #[clap(short = 'D', long, env, hide_env = true)]
    pub data: Option<String>,
    /// Headers to send
    #[clap(short = 'H', long, value_name = "key:value", value_parser = is_header, env, hide_env=true)]
    pub headers: Vec<String>,
    /// Cookies to send
    #[clap(short, long, value_name = "key=value", value_parser = is_cookie, env, hide_env=true)]
    pub cookies: Vec<String>,
    /// Follow redirects
    #[clap(
        short = 'R',
        long,
        default_value = "2",
        value_name = "COUNT",
        env,
        hide_env = true
    )]
    pub follow_redirects: Option<usize>,
    /// Request throttling (requests per second) per thread
    #[clap(long, default_value = "0", env, hide_env = true)]
    pub throttle: Option<usize>,
    /// Max time to run (will abort after given time) in seconds
    #[clap(short = 'M', long, env, hide_env = true)]
    pub max_time: Option<usize>,
    /// Don't use colors
    /// You can also set the NO_COLOR environment variable
    #[clap(long, alias = "no-colors", env, hide_env = true)]
    pub no_color: bool,
    /// Quiet mode
    #[clap(short, long, env, hide_env = true)]
    pub quiet: bool,
    /// Interactive mode
    #[clap(short, long, env, hide_env = true)]
    pub interactive: bool,
    /// Insecure mode, disables SSL certificate validation
    #[clap(long, env, hide_env = true)]
    pub insecure: bool,
    /// Show response additional body information: "length", "hash", "headers_length", "headers_hash", "body", "headers"
    #[clap(long, env, hide_env = true)]
    pub show: Vec<String>,

    /// Resume from a saved file
    #[clap(short='r', long, help_heading = Some("Resume"), env, hide_env=true)]
    pub resume: bool,
    /// Custom save file
    #[clap(long, default_value = SAVE_FILE, help_heading = Some("Resume"), value_name = "FILE", env, hide_env=true)]
    pub save_file: String,
    /// Don't save the state in case you abort
    #[clap(long, help_heading = Some("Resume"), env, hide_env=true)]
    pub no_save: bool,

    /// Wordlist transformations: "lower", "upper", "prefix", "suffix", "capitalize", "reverse", "remove", "replace"
    #[clap(short='T', long, help_heading = Some("Transformations"), env, hide_env=true, value_parser(parse_key_or_key_val::<String, String>))]
    pub transform: Vec<(String, Option<String>)>,

    /// Wordlist filtering: "contains", "starts", "ends", "regex", "length"
    #[clap(short='w', long, help_heading = Some("Filtering"), value_name = "KEY:FILTER", env, hide_env=true, value_parser(parse_key_val::<String, String>), visible_alias = "wf")]
    pub wordlist_filter: Vec<(String, String)>,

    /// Response filtering: "time", "status", "contains", "starts", "end", "regex", "length", "hash"
    #[clap(short, long, help_heading = Some("Filtering"), value_name = "KEY:FILTER", env, hide_env=true, value_parser(parse_key_val::<String, String>))]
    pub filter: Vec<(String, String)>,
    /// Treat filters as or instead of and
    #[clap(long, help_heading = Some("Filtering"), env, hide_env=true)]
    pub or: bool,

    /// Proxy URL
    #[clap(short='P', long, help_heading = Some("Proxy"), value_name = "URL", env, hide_env=true)]
    pub proxy: Option<String>,
    /// Proxy username and password
    #[clap(long, help_heading = Some("Proxy"), value_name = "USER:PASS", env, hide_env=true)]
    pub proxy_auth: Option<String>,

    /// Generate markdown help - for developers
    #[clap(long, hide = true)]
    pub generate_markdown: bool,
}

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find(':')
        .ok_or_else(|| format!("invalid KEY:value: no `:` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn parse_key_or_key_val<T, U>(
    s: &str,
) -> Result<(T, Option<U>), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    if s.contains(':') {
        let pos = s
            .find(':')
            .ok_or_else(|| format!("invalid KEY:value: no `:` found in `{s}`"))?;
        Ok((s[..pos].parse()?, Some(s[pos + 1..].parse()?)))
    } else {
        Ok((s.parse()?, None))
    }
}

fn parse_cli_range_input(s: &str) -> Result<String> {
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

pub async fn main_interactive() -> Result<()> {
    let mut rl = rustyline::DefaultEditor::new()?;
    let mut state = Opts::parse();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let parts = line.split(" ").collect::<Vec<_>>();
                let cmd = parts[0];
                if cmd.is_empty() {
                    continue;
                }
                let args = parts[1..].to_vec();
                // This is a bit ugly, but I can't manage to box async functions
                match cmd {
                    "help" | "h" | "?" => help(&mut rl, args, &mut state).await,
                    "exit" | "quit" | "q" => exit(&mut rl, args, &mut state).await,
                    "clear" | "cls" => clear(&mut rl, args, &mut state).await,
                    "set" | "s" => set(&mut rl, args, &mut state).await,
                    "append" | "a" => append(&mut rl, args, &mut state).await,
                    "unset" | "u" => unset(&mut rl, args, &mut state).await,
                    "get" | "g" => get(&mut rl, args, &mut state).await,
                    "list" | "ls" | "l" => list(&mut rl, args, &mut state).await,
                    "run" | "r" => run(&mut rl, args, &mut state).await,
                    _ => {
                        println!("Unknown command: {}", cmd);
                        Ok(())
                    }
                }?;
            }
            Err(_) => break,
        }
    }
    Ok(())
}

struct Command {
    name: String,
    aliases: Vec<String>,
    description: String,
}

impl Command {
    fn new(name: &str, aliases: Vec<&str>, description: &str) -> Self {
        Self {
            name: name.to_string(),
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
            description: description.to_string(),
        }
    }
}

lazy_static! {
    static ref COMMANDS: Vec<Command> = vec![
        Command::new("help", vec!["h", "?"], "Show this help message"),
        Command::new("exit", vec!["quit", "q"], "Exit the program"),
        Command::new("clear", vec!["cls"], "Clear the screen"),
        Command::new("set", vec!["s"], "Set a value"),
        Command::new("unset", vec!["u"], "Unset a value"),
        Command::new("get", vec!["g"], "Get a value"),
        Command::new("list", vec!["ls"], "List all values"),
        Command::new("run", vec!["r"], "Run the scanner"),
    ];
}

async fn help(_rl: &mut DefaultEditor, _args: Vec<&str>, _state: &mut Opts) -> Result<()> {
    println!("Available commands:");
    for cmd in COMMANDS.iter() {
        println!("  {:<10} {}", cmd.name.bold(), cmd.description.dimmed());
    }
    Ok(())
}

async fn exit(_rl: &mut DefaultEditor, _args: Vec<&str>, _state: &mut Opts) -> Result<()> {
    std::process::exit(0);
}

async fn clear(rl: &mut DefaultEditor, _args: Vec<&str>, _state: &mut Opts) -> Result<()> {
    rl.clear_screen()?;
    Ok(())
}

async fn set(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut Opts) -> Result<()> {
    if args.len() != 2 {
        println!("Usage: set <key> <value>");
        return Ok(());
    }
    let key = args[0];
    let value = args[1];
    let value_type = get_value_type(value);

    let res = match value_type {
        ValueType::String => state.set(&key.to_string(), Some(value.to_string())),
        ValueType::Bool => state.set(&key.to_string(), parse_bool(value)),
        ValueType::Usize => state.set(&key.to_string(), Some(value.parse::<usize>().unwrap())),
        ValueType::Range => state.set(
            &key.to_string(),
            Some(parse_cli_range_input(value).unwrap()),
        ),
        ValueType::StringVec => {
            let re = regex::Regex::new(r#"\[(.*)\]"#).unwrap();
            let value = re.replace_all(value, "$1").to_string();
            let value = value.split(",").map(|s| s.to_string()).collect::<Vec<_>>();
            state.set(&key.to_string(), value)
        }
    };

    match res {
        Ok(_) => {}
        Err(_) => {
            // Try to set the value as a string
            let res = state.set(&key.to_string(), Some(value.to_string()));
            match res {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {}", e);
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

async fn append(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut Opts) -> Result<()> {
    if args.len() != 2 {
        println!("Usage: append <key> <value>");
        return Ok(());
    }
    let key = args[0];
    let value = args[1];

    let re = regex::Regex::new(r#"\[(.*)\]"#).unwrap();
    let current_value = match state.getenum(&key.to_string()) {
        Ok(value) => {
            let s = format!("{:?}", value);
            // depth(Some(1)) or depth(None)
            let re = regex::Regex::new(format!(r#"{}\(Some\((.*)\)\)"#, key).as_str()).unwrap();
            let s = re.replace_all(&s, "$1").to_string();
            // depth(1)
            let re = regex::Regex::new(format!(r#"{}\((.*)\)"#, key).as_str()).unwrap();
            let s = re.replace_all(&s, "$1").to_string();
            s
        }
        Err(_) => "".to_string(),
    };
    let current_value = re.replace_all(&current_value, "$1").to_string();
    let mut current_value = current_value
        .split(",")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let mut value = value.split(",").map(|s| s.to_string()).collect::<Vec<_>>();
    current_value.append(&mut value);
    let res = state.set(
        &key.to_string(),
        current_value
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    match res {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    }

    Ok(())
}

#[derive(Debug)]
enum ValueType {
    String,
    Bool,
    Usize,
    Range,
    StringVec,
}

fn parse_bool(s: &str) -> bool {
    match s.to_lowercase().as_str() {
        "true" => true,
        "false" => false,
        _ => false,
    }
}

fn get_value_type(s: &str) -> ValueType {
    if s.starts_with("[") && s.ends_with("]") {
        return ValueType::StringVec;
    }
    if ["true", "false"].contains(&s.to_lowercase().as_str()) {
        return ValueType::Bool;
    }
    if s.parse::<usize>().is_ok() {
        return ValueType::Usize;
    }
    if parse_cli_range_input(s).is_ok() {
        return ValueType::Range;
    }
    ValueType::String
}

async fn unset(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut Opts) -> Result<()> {
    if args.len() != 1 {
        println!("Usage: unset <key>");
        return Ok(());
    }
    let key = args[0];
    let res = state.set(&key.to_string(), None as Option<String>);
    match res {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    }

    Ok(())
}

async fn get(_rl: &mut DefaultEditor, args: Vec<&str>, state: &mut Opts) -> Result<()> {
    if args.len() != 1 {
        println!("Usage: get <key>");
        return Ok(());
    }
    let key = args[0];
    let value = match state.getenum(&key.to_string()) {
        Ok(value) => {
            let s = format!("{:?}", value);
            // depth(Some(1)) or depth(None)
            let re = regex::Regex::new(format!(r#"{}\(Some\((.*)\)\)"#, key).as_str()).unwrap();
            let s = re.replace_all(&s, "$1").to_string();
            // depth(1)
            let re = regex::Regex::new(format!(r#"{}\((.*)\)"#, key).as_str()).unwrap();
            let s = re.replace_all(&s, "$1").to_string();
            s
        }
        Err(_) => {
            println!("Unknown key: {}", key);
            return Ok(());
        }
    };

    println!("{}: {}", key, value);
    Ok(())
}

async fn list(_rl: &mut DefaultEditor, _args: Vec<&str>, state: &mut Opts) -> Result<()> {
    println!("Listing all values:");
    let struct_info = state.getstructinfo();
    let mut fields: Vec<(String, String, String)> =
        vec![("".to_string(), "".to_string(), "".to_string()); struct_info.field_names.len()];
    for (i, name) in struct_info.field_names.iter().enumerate() {
        fields[i].0 = name.to_string();
        let value = match state.getenum(&name.to_string()) {
            Ok(value) => {
                let s = format!("{:?}", value);
                // depth(Some(1)) or depth(None)
                let re =
                    regex::Regex::new(format!(r#"{}\(Some\((.*)\)\)"#, name).as_str()).unwrap();
                let s = re.replace_all(&s, "$1").to_string();
                // depth(1)
                let re = regex::Regex::new(format!(r#"{}\((.*)\)"#, name).as_str()).unwrap();
                let s = re.replace_all(&s, "$1").to_string();
                s
            }
            Err(_) => "".to_string(),
        };
        fields[i].2 = value;
    }
    for (i, ty) in struct_info.field_types.iter().enumerate() {
        let re = regex::Regex::new(r#"Option < (.*) >"#).unwrap();
        let ty = re.replace_all(ty, "$1").to_string();
        let re = regex::Regex::new(r#"Vec < (.*) >"#).unwrap();
        let ty = re.replace_all(&ty, "Vec<$1>").to_string();
        fields[i].1 = ty;
    }

    let max_len = fields
        .iter()
        .map(|(name, _, _)| name.len())
        .max()
        .unwrap_or(0);

    for (name, ty, value) in fields {
        println!(
            "  {:<width$} {} = {}",
            name.bold(),
            ty.dimmed(),
            value,
            width = max_len,
        );
    }
    Ok(())
}

async fn run(_rl: &mut DefaultEditor, _args: Vec<&str>, state: &mut Opts) -> Result<()> {
    let _ = crate::_main(state.clone()).await;
    Ok(())
}
