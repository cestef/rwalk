use crate::{
    Result,
    error::{SyntaxError, syntax_error},
    utils::{constants::DEFAULT_WORDLIST_KEY, types::ThrottleMode},
};
use clap::ValueEnum;
use dashmap::DashSet as HashSet;

pub fn parse_url(s: &str) -> Result<url::Url> {
    // Replace google.com with http://google.com
    let s = if s.starts_with("http://") || s.starts_with("https://") {
        s.to_string()
    } else {
        format!("http://{}", s)
    };

    url::Url::parse(&s).map_err(|e| syntax_error!((0, s.len()), s, "{e}"))
}

// Parse [key1,key2]name:value or name:value
pub fn parse_keyed_keyval(s: &str) -> Result<(HashSet<String>, String, String)> {
    let (keys, rest) = parse_optional_keys(s)?;
    let (name, value) = parse_keyval(rest)?;
    Ok((keys, name, value))
}

// Parse [key1,key2]name[:value] or name[:value]
pub fn parse_keyed_key_or_keyval(s: &str) -> Result<(HashSet<String>, String, Option<String>)> {
    let (keys, rest) = parse_optional_keys(s)?;
    let (name, value) = parse_key_or_keyval(rest)?;
    Ok((keys, name, value))
}

// Helper to parse optional [key1,key2] prefix
pub fn parse_optional_keys(s: &str) -> Result<(HashSet<String>, &str)> {
    if s.starts_with('[') {
        if let Some(end_bracket) = s.find(']') {
            let keys: HashSet<String> = s[1..end_bracket]
                .split(',')
                .map(|k| k.trim().to_string())
                .collect();
            if keys.is_empty() {
                return Err(syntax_error!((0, end_bracket + 1), s, "Empty key list"));
            }
            Ok((keys, &s[end_bracket + 1..]))
        } else {
            Err(syntax_error!((0, s.len()), s, "Unclosed '[' bracket"))
        }
    } else {
        // No keys specified - use empty set to indicate "applies to all"
        Ok((HashSet::new(), s))
    }
}

// key:value
pub fn parse_keyval_with_sep(s: &str, sep: char) -> Result<(String, String)> {
    let parts: Vec<&str> = s.split(sep).collect();
    if parts.len() != 2 {
        return Err(syntax_error!((0, s.len()), s, "Expected exactly one ':'"));
    }
    Ok((parts[0].to_lowercase(), parts[1].to_string()))
}

pub fn parse_keyval(s: &str) -> Result<(String, String)> {
    parse_keyval_with_sep(s, ':')
}

// key[:value]
pub fn parse_key_or_keyval(s: &str) -> Result<(String, Option<String>)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() > 2 {
        return Err(syntax_error!((0, s.len()), s, "Expected at most one ':'"));
    }
    Ok((parts[0].to_lowercase(), parts.get(1).map(|s| s.to_string())))
}

pub fn parse_wordlist(s: &str) -> Result<(String, String)> {
    let res = parse_key_or_keyval(s)?;
    let res = (
        res.0,
        res.1.unwrap_or_else(|| DEFAULT_WORDLIST_KEY.to_string()),
    );
    Ok(res)
}

enum Throttle {
    Range(u64, u64),
    Auto,
}

// auto or max[:mode]
// mode simple or dynamic
// default is simple
pub fn parse_throttle(s: &str) -> Result<(u64, ThrottleMode)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() > 2 {
        return Err(syntax_error!((0, s.len()), s, "Expected at most one ':'"));
    }
    let max = parts[0].parse::<u64>()?;
    let mode = if parts.len() == 2 {
        ThrottleMode::from_str(parts[1], true)
            .map_err::<crate::RwalkError, _>(|e| syntax_error!((0, s.len()), s, "{e}"))?
    } else {
        ThrottleMode::Simple
    };
    Ok((max, mode))
}

fn parse_throttle_range(s: &str) -> Result<(u64, u64)> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        1 => {
            let max = parts[0].parse()?;
            Ok((1, max))
        }
        2 => {
            let min = parts[0].parse()?;
            let max = parts[1].parse()?;
            Ok((min, max))
        }
        _ => Err(syntax_error!((0, s.len()), s, "Expected at most one ':'")),
    }
}

// @file or raw string
pub fn parse_filter(s: &str) -> Result<String> {
    if let Some(path) = s.strip_prefix('@') {
        let content = std::fs::read_to_string(path)?;
        Ok(content)
    } else {
        Ok(s.to_string())
    }
}
