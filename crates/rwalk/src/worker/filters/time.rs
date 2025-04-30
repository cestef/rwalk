use crate::{error, utils::types::IntRange};

use super::response_filter;

response_filter!(
    TimeFilter,
    Vec<IntRange<u64>>,
    needs_body = false,
    |res: &RwalkResponse, range: &Vec<IntRange<u64>>| Ok(range
        .iter()
        .any(|r| r.contains((res.time / 1_000_000) as u64))),
    "time",
    ["elapsed", "duration", "d"],
    transform = |raw: String| raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|s| { IntRange::from_str_with_mapper(&s, Some(parse_duration_microseconds)) })
        .collect::<Result<_>>()
);
use crate::RwalkError;

// us, ms, s, m
fn parse_duration_microseconds(raw: &str) -> Result<u64> {
    let mut parts = raw.split_whitespace();
    let value = parts
        .next()
        .ok_or_else(|| error!("Missing number"))?
        .parse::<u64>()
        .map_err(|_| error!("Invalid number"))?;
    let unit = parts.next().ok_or_else(|| error!("Missing unit"))?;

    let multiplier = match unit {
        "us" => 1,
        "ms" => 1_000,
        "s" => 1_000_000,
        "m" => 60 * 1_000_000,
        _ => return Err(error!("Invalid unit")),
    };

    Ok(value * multiplier)
}
