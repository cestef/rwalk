use clap::Parser;
use lazy_static::lazy_static;
use url::Url;
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
    #[clap(short, long)]
    pub output: Option<String>,
    /// Request timeout in seconds
    #[clap(short = 'T', long, default_value = "5")]
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
    /// Data to send
    #[clap(short, long)]
    pub data: Option<String>,
    /// Headers to send
    #[clap(short = 'H', long, value_name = "key:value", value_parser = is_header)]
    pub headers: Vec<String>,
    /// Cookies to send
    #[clap(short, long, value_name = "key=value", value_parser = is_cookie)]
    pub cookies: Vec<String>,
    /// Case insensitive
    #[clap(short = 'I', long)]
    pub case_insensitive: bool,
    /// Follow redirects
    #[clap(short = 'F', long, default_value = "0")]
    pub follow_redirects: usize,
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
