use std::path::PathBuf;

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
                .filter_map(|k| {
                    let k = k.trim();
                    if k.is_empty() {
                        None
                    } else {
                        Some(k.to_string())
                    }
                })
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

// key1,key2:val
pub fn parse_multikey_val(s: &str) -> Result<(HashSet<String>, String)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(syntax_error!((0, s.len()), s, "Expected exactly one ':'"));
    }
    let keys: HashSet<String> = parts[0]
        .split(',')
        .filter_map(|k| {
            let k = k.trim();
            if k.is_empty() {
                None
            } else {
                Some(k.to_string())
            }
        })
        .collect();
    if keys.is_empty() {
        return Err(syntax_error!((0, s.len()), s, "Empty key list"));
    }
    Ok((keys, parts[1].to_string()))
}

// key[:value]
pub fn parse_key_or_keyval(s: &str) -> Result<(String, Option<String>)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() > 2 {
        return Err(syntax_error!((0, s.len()), s, "Expected at most one ':'"));
    }
    let key = parts[0].to_string(); // ← no lowercase here
    let value = parts.get(1).map(|s| s.to_string());
    Ok((key, value))
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

// key[:path]
pub fn parse_save_wordlist(s: &str) -> Result<(String, Option<PathBuf>)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() > 2 {
        return Err(syntax_error!((0, s.len()), s, "Expected at most one ':'"));
    }

    let key = parts[0].to_string();
    let path = if parts.len() == 2 {
        Some(PathBuf::from(parts[1]))
    } else {
        None
    };

    Ok((key, path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RwalkError;

    #[test]
    fn test_parse_url() {
        // Valid URLs
        let url = parse_url("google.com").unwrap();
        assert_eq!(url.as_str(), "http://google.com/");

        let url = parse_url("http://google.com").unwrap();
        assert_eq!(url.as_str(), "http://google.com/");

        let url = parse_url("https://google.com").unwrap();
        assert_eq!(url.as_str(), "https://google.com/");

        // Invalid URL
        let err = parse_url("http://[").unwrap_err();
        assert!(matches!(err, RwalkError::SyntaxError(_)));
    }

    #[test]
    fn test_parse_optional_keys() {
        // No keys
        let (keys, rest) = parse_optional_keys("name:value").unwrap();
        assert_eq!(keys.len(), 0);
        assert_eq!(rest, "name:value");

        // With keys
        let (keys, rest) = parse_optional_keys("[key1,key2]name:value").unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains("key1"));
        assert!(keys.contains("key2"));
        assert_eq!(rest, "name:value");

        // Empty keys
        let err = parse_optional_keys("[]name:value").unwrap_err();
        assert!(matches!(err, RwalkError::SyntaxError(_)));

        // Unclosed bracket
        let err = parse_optional_keys("[key1,key2name:value").unwrap_err();
        assert!(matches!(err, RwalkError::SyntaxError(_)));
    }

    #[test]
    fn test_parse_keyval() {
        // Valid key-value pair
        let (key, val) = parse_keyval("name:value").unwrap();
        assert_eq!(key, "name");
        assert_eq!(val, "value");

        // Missing colon
        let err = parse_keyval("namevalue").unwrap_err();
        assert!(matches!(err, RwalkError::SyntaxError(_)));

        // Multiple colons
        let err = parse_keyval("name:val:ue").unwrap_err();
        assert!(matches!(err, RwalkError::SyntaxError(_)));
    }

    #[test]
    fn test_parse_key_or_keyval() {
        // Key only
        let (key, val) = parse_key_or_keyval("name").unwrap();
        assert_eq!(key, "name");
        assert_eq!(val, None);

        // Key-value pair
        let (key, val) = parse_key_or_keyval("name:value").unwrap();
        assert_eq!(key, "name");
        assert_eq!(val, Some("value".to_string()));

        // Multiple colons
        let err = parse_key_or_keyval("name:val:ue").unwrap_err();
        assert!(matches!(err, RwalkError::SyntaxError(_)));
    }

    #[test]
    fn test_parse_keyed_keyval() {
        // No keys
        let (keys, name, value) = parse_keyed_keyval("name:value").unwrap();
        assert_eq!(keys.len(), 0);
        assert_eq!(name, "name");
        assert_eq!(value, "value");

        // With keys
        let (keys, name, value) = parse_keyed_keyval("[key1,key2]name:value").unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains("key1"));
        assert!(keys.contains("key2"));
        assert_eq!(name, "name");
        assert_eq!(value, "value");

        // Invalid format
        let err = parse_keyed_keyval("namevalue").unwrap_err();
        assert!(matches!(err, RwalkError::SyntaxError(_)));
    }

    #[test]
    fn test_parse_keyed_key_or_keyval() {
        // No keys, key only
        let (keys, name, value) = parse_keyed_key_or_keyval("name").unwrap();
        assert_eq!(keys.len(), 0);
        assert_eq!(name, "name");
        assert_eq!(value, None);

        // No keys, key-value pair
        let (keys, name, value) = parse_keyed_key_or_keyval("name:value").unwrap();
        assert_eq!(keys.len(), 0);
        assert_eq!(name, "name");
        assert_eq!(value, Some("value".to_string()));

        // With keys, key only
        let (keys, name, value) = parse_keyed_key_or_keyval("[key1,key2]name").unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(name, "name");
        assert_eq!(value, None);

        // With keys, key-value pair
        let (keys, name, value) = parse_keyed_key_or_keyval("[key1,key2]name:value").unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(name, "name");
        assert_eq!(value, Some("value".to_string()));
    }

    #[test]
    fn test_parse_wordlist() {
        // With explicit key
        let (name, key) = parse_wordlist("wordlist:common").unwrap();
        assert_eq!(name, "wordlist");
        assert_eq!(key, "common");

        // Default key
        let (name, key) = parse_wordlist("wordlist").unwrap();
        assert_eq!(name, "wordlist");
        assert_eq!(key, DEFAULT_WORDLIST_KEY);
    }

    #[test]
    fn test_parse_throttle() {
        // Default mode
        let (max, mode) = parse_throttle("100").unwrap();
        assert_eq!(max, 100);
        assert_eq!(mode, ThrottleMode::Simple);

        // Explicit mode
        let (max, mode) = parse_throttle("100:dynamic").unwrap();
        assert_eq!(max, 100);
        assert_eq!(mode, ThrottleMode::Dynamic);

        // Invalid mode
        let err = parse_throttle("100:invalid").unwrap_err();
        assert!(matches!(err, RwalkError::SyntaxError(_)));

        // Invalid number
        let err = parse_throttle("abc").unwrap_err();
        assert!(matches!(err, RwalkError::ParseError(_)));
    }

    #[test]
    fn test_parse_throttle_range() {
        // Single value (min=1)
        let (min, max) = parse_throttle_range("100").unwrap();
        assert_eq!(min, 1);
        assert_eq!(max, 100);

        // Min and max
        let (min, max) = parse_throttle_range("50:100").unwrap();
        assert_eq!(min, 50);
        assert_eq!(max, 100);

        // Multiple colons
        let err = parse_throttle_range("10:20:30").unwrap_err();
        assert!(matches!(err, RwalkError::SyntaxError(_)));

        // Invalid number
        let err = parse_throttle_range("abc").unwrap_err();
        assert!(matches!(err, RwalkError::ParseError(_)));
    }

    #[test]
    fn test_parse_filter() {
        // Raw string
        let filter = parse_filter("some filter content").unwrap();
        assert_eq!(filter, "some filter content");

        let filter = parse_filter("@tests/assets/dummy.txt").unwrap();
        assert_eq!(
            filter,
            include_str!("../../tests/assets/dummy.txt").to_string()
        );
    }
}
