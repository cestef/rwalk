use std::{borrow::Cow, fmt::Display};

use owo_colors::{OwoColorize, Rgb};

use crate::worker::utils::RwalkResponse;

pub fn response(response: &RwalkResponse) -> String {
    format!(
        "{} {} {}",
        display_status_code(response.status),
        display_url(response.url.as_str()),
        display_time(response.time.as_nanos())
    )
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

fn display_status_code(s: u16) -> String {
    format!(
        "{} {}",
        color_for_status_code(icon_for_status_code(s).to_string(), s),
        s.dimmed()
    )
}

fn color_for_status_code(icon: String, s: u16) -> String {
    match s {
        100..=199 => icon.blue().to_string(),
        200..=299 => icon.green().to_string(),
        300..=399 => icon.yellow().to_string(),
        400..=499 => icon.color(Rgb(255, 165, 0)).to_string(),
        500..=599 => icon.red().to_string(),
        _ => icon.to_string(),
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

macro_rules! success {
    ($($arg:tt)*) => {
        println!("{} {}", "✓".green(), format!($($arg)*))
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        println!("{} {}", "✖".red(), format!($($arg)*))
    };
}

macro_rules! warning {
    ($($arg:tt)*) => {
        println!("{} {}", "⚠".yellow(), format!($($arg)*))
    };
}

macro_rules! info {
    ($($arg:tt)*) => {
        println!("{} {}", "ℹ️".blue(), format!($($arg)*))
    };
}

// pub(crate) use error;
// pub(crate) use info;
pub(crate) use success;
pub(crate) use warning;

pub enum SkipReason {
    NonDirectory,
}

impl Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkipReason::NonDirectory => write!(f, "is not a directory"),
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
