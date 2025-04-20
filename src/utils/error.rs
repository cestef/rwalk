use miette::{Diagnostic, SourceSpan};
use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum RwalkError {
    #[error(transparent)]
    #[diagnostic(code(rwalk::io_error))]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    SyntaxError(#[from] SyntaxError),

    #[diagnostic(code(rwalk::error))]
    #[error("{message}")]
    Error {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[diagnostic(code(rwalk::http_error))]
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),

    #[diagnostic(code(rwalk::thread_error))]
    #[error(transparent)]
    ThreadError(#[from] tokio::task::JoinError),

    #[diagnostic(code(rwalk::url_error))]
    #[error(transparent)]
    UrlError(#[from] url::ParseError),

    #[diagnostic(code(rwalk::parse_error))]
    #[error(transparent)]
    ParseError(#[from] std::num::ParseIntError),

    #[diagnostic(code(rwalk::json_error))]
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[diagnostic(code(rwalk::regex_error))]
    #[error(transparent)]
    RegexError(#[from] regex::Error),

    #[diagnostic(code(rwalk::unreachable_host))]
    #[error("Host is unreachable, use --force to ignore")]
    UnreachableHost {
        #[source]
        source: reqwest::Error,
    },

    #[diagnostic(code(rwalk::toml_error))]
    #[error(transparent)]
    TomlError(#[from] toml::de::Error),

    #[diagnostic(code(rwalk::clap_error))]
    #[error(transparent)]
    ClapError(#[from] clap::Error),

    #[diagnostic(code(rwalk::rhai_error))]
    #[error("Script error: {0}")]
    RhaiError(String),

    #[diagnostic(code(rwalk::invalid_header_value))]
    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[diagnostic(code(rwalk::invalid_header_name))]
    #[error(transparent)]
    InvalidHeaderName(#[from] InvalidHeaderName),

    #[diagnostic(code(rwalk::readline_error))]
    #[error(transparent)]
    ReadlineError(#[from] rustyline::error::ReadlineError),
}

#[derive(Error, Diagnostic, Debug, Clone)]
#[error("Syntax error: {message}")]
#[diagnostic(code(rwalk::syntax_error))]
pub struct SyntaxError {
    #[source_code]
    pub src: String,
    #[label("right here")]
    pub span: SourceSpan,
    pub message: String,
}

pub type Result<T, U = RwalkError> = std::result::Result<T, U>;

macro_rules! error {
    (source = $source:expr, $($arg:tt)*) => {
        (RwalkError::Error {
            message: format!($($arg)*),
            source: Some(Box::new($source)),
        })
    };
    ($($arg:tt)*) => {
        (RwalkError::Error {
            message: format!($($arg)*),
            source: None,
        })
    };
}

macro_rules! syntax_error {
    ($span:expr, $src:expr, $($arg:tt)*) => {
        (SyntaxError {
            span: $span.into(),
            src: $src.to_string(),
            message: format!($($arg)*),
        }).into()
    };
}

pub(crate) use error;
pub(crate) use syntax_error;
