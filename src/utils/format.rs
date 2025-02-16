use std::{borrow::Cow, fmt::Display};

use owo_colors::OwoColorize;

use crate::worker::utils::RwalkResponse;

pub fn response(response: &RwalkResponse, show: &Vec<String>) -> String {
    format!(
        "{} {} {} {}",
        display_status_code(response.status),
        display_url(response.url.as_str()),
        display_time(response.time.as_nanos()),
        display_show(response, show)
    )
}

fn display_show(response: &RwalkResponse, show: &Vec<String>) -> String {
    let mut show = show.iter().map(|s| s.to_lowercase());
    let mut output = String::new();
    while let Some(s) = show.next() {
        match s.as_str() {
            "size" => output.push_str(&response.body.as_ref().map_or(0, |e| e.len()).to_string()),
            _ => {}
        }
    }
    output
}

fn display_url(url: &str) -> Cow<'_, str> {
    urlencoding::decode(url).unwrap_or(url.into())
}

pub fn display_time(t: u128) -> String {
    let t = t as f64 / 1_000_000.0;
    let mut unit: &str = "ms";
    let mut value: f64 = t;
    if t < 1.0 {
        unit = "μs";
        value = t * 1_000.0;
    } else if t >= 1_000.0 {
        unit = "s";
        value = t / 1_000.0;
    }

    format!("{:.2}{}", value.dimmed().bold(), unit.dimmed())
}

pub fn display_status_code(s: u16) -> String {
    format!(
        "{} {}",
        color_for_status_code(icon_for_status_code(s), s),
        s.dimmed()
    )
}

pub fn color_for_status_code(input: &str, s: u16) -> String {
    match s {
        100..=199 => input.blue().to_string(),
        200..=299 => input.green().to_string(),
        300..=399 => input.blue().to_string(),
        400..=499 => input.yellow().to_string(),
        500..=599 => input.red().to_string(),
        _ => input.to_string(),
    }
}

fn icon_for_status_code(s: u16) -> &'static str {
    match s {
        100..=199 => "ℹ️",
        200..=299 => "✓",
        300..=399 => "🔀",
        400..=499 => "⚠",
        500..=599 => "✖",
        _ => "❓",
    }
}

// pub fn warning(msg: &str) -> String {
//     format!("{} {}", "⚠".yellow(), msg)
// }

// pub fn error(msg: &str) -> String {
//     format!("{} {}", "✖".red(), msg)
// }

// pub fn info(msg: &str) -> String {
//     format!("{} {}", "ℹ️".blue(), msg)
// }

// pub fn success(msg: &str) -> String {
//     format!("{} {}", "✓".green(), msg)
// }

pub const WARNING: &str = "⚠";
pub const ERROR: &str = "✖";
pub const INFO: &str = "ℹ";
pub const SUCCESS: &str = "✓";

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        println!("{} {}", "✓".green(), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        println!("{} {}", "✖".red(), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {
        println!("{} {}", "⚠".yellow(), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("{} {}", "ℹ".blue(), format!($($arg)*))
    };
}

// pub(crate) use info;
// pub(crate) use print_error;
// pub(crate) use success;
// pub(crate) use warning;

pub enum SkipReason {
    NonDirectory,
    MaxDepth,
}

impl Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkipReason::NonDirectory => write!(f, "not a directory"),
            SkipReason::MaxDepth => write!(f, "max depth reached"),
        }
    }
}

pub fn skip(response: &RwalkResponse, reason: SkipReason) -> String {
    format!(
        "{} {} {} {} {}",
        "↷".blue(),
        response.status.dimmed(),
        display_url(response.url.as_str()),
        display_time(response.time.as_nanos()),
        format!("({})", reason).dimmed()
    )
}
