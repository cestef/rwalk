use crate::{
    error::{syntax_error, SyntaxError},
    utils::constants::DEFAULT_WORDLIST_KEY,
    Result,
};
use dashmap::DashSet as HashSet;

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
pub fn parse_keyval(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(syntax_error!((0, s.len()), s, "Expected exactly one ':'"));
    }
    Ok((parts[0].to_lowercase(), parts[1].to_string()))
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

// <min>:<max> -> (min, max) or <max> -> (0, max)
pub fn parse_throttle(s: &str) -> Result<(u64, u64)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() == 1 {
        let max = parts[0].parse()?;
        return Ok((1, max));
    } else if parts.len() == 2 {
        let min = parts[0].parse()?;
        let max = parts[1].parse()?;
        return Ok((min, max));
    }
    Err(syntax_error!((0, s.len()), s, "Expected exactly one ':'"))
}
