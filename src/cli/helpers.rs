use std::fmt::Display;

use super::opts::Wordlist;
use clap::{
    builder::TypedValueParser,
    error::{ContextKind, ContextValue, ErrorKind},
};
use serde::{Deserialize, Serialize};
use tabled::Tabled;
use url::Url;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Tabled)]
pub struct KeyVal<T: Display, U: Display>(
    #[tabled(rename = "Key")] pub T,
    #[tabled(rename = "Value")] pub U,
);

impl<T: Display, U: Display> Display for KeyVal<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[derive(Clone)]
pub struct KeyValParser;

impl TypedValueParser for KeyValParser {
    type Value = KeyVal<String, String>;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let s = value.to_string_lossy();
        let pos = s
            .find(':')
            .ok_or_else(|| format!("invalid KEY:value: no `:` found in `{s}`"))
            .map_err(|_| {
                let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
                if let Some(arg) = arg {
                    err.insert(
                        ContextKind::InvalidArg,
                        ContextValue::String(arg.to_string()),
                    );
                }
                err.insert(
                    ContextKind::InvalidValue,
                    ContextValue::String(s.to_string()),
                );

                Err(err)
            });
        if let Err(e) = pos {
            e
        } else {
            let pos = pos.unwrap();
            Ok(KeyVal(s[..pos].to_string(), s[pos + 1..].to_string()))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct KeyOrKeyVal<T: Display, U: Display>(pub T, pub Option<U>);

impl Display for KeyOrKeyVal<String, String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.1 {
            Some(v) => write!(f, "{}:{}", self.0, v),
            None => write!(f, "{}", self.0),
        }
    }
}
#[derive(Clone)]
pub struct KeyOrKeyValParser;

impl TypedValueParser for KeyOrKeyValParser {
    type Value = KeyOrKeyVal<String, String>;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let s = value.to_string_lossy();
        if s.contains(':') {
            let pos = s
                .find(':')
                .ok_or_else(|| format!("invalid KEY:value: no `:` found in `{s}`"))
                .map_err(|_| {
                    let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
                    if let Some(arg) = arg {
                        err.insert(
                            ContextKind::InvalidArg,
                            ContextValue::String(arg.to_string()),
                        );
                    }
                    err.insert(
                        ContextKind::InvalidValue,
                        ContextValue::String(s.to_string()),
                    );

                    Err(err)
                });
            if let Err(e) = pos {
                e
            } else {
                let pos = pos.unwrap();
                Ok(KeyOrKeyVal(
                    s[..pos].to_string(),
                    Some(s[pos + 1..].to_string()),
                ))
            }
        } else {
            Ok(KeyOrKeyVal(s.to_string(), None))
        }
    }
}

pub fn parse_url(s: &str) -> Result<String, String> {
    let s = if !s.starts_with("http://") && !s.starts_with("https://") {
        format!("http://{}", s)
    } else {
        s.to_string()
    };
    let url = Url::parse(&s);

    match url {
        Ok(url) => {
            if url.host().is_none() {
                return Err("Invalid URL".to_string());
            }
            Ok(url.to_string())
        }
        Err(_) => Err("Invalid URL".to_string()),
    }
}

pub fn parse_header(s: &str) -> Result<String, String> {
    // key: value
    let parts = s.split(':').collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err("Invalid header".to_string());
    }
    Ok(s.to_string())
}

pub fn parse_cookie(s: &str) -> Result<String, String> {
    // key=value
    let parts = s.split('=').collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err("Invalid cookie".to_string());
    }
    Ok(s.to_string())
}

pub fn parse_method(s: &str) -> Result<String, String> {
    let methods = [
        "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "TRACE", "CONNECT",
    ];
    let s = s.to_uppercase();
    if methods.contains(&s.as_str()) {
        Ok(s.to_string())
    } else {
        Err("Invalid HTTP method".to_string())
    }
}

pub fn parse_wordlist(s: &str) -> Result<Wordlist, String> {
    let parts = s.split(':').collect::<Vec<_>>();
    if parts.len() == 1 {
        // Wordlist without a key
        Ok(Wordlist(s.to_string(), vec![]))
    } else if parts.len() == 2 {
        // Wordlist with a key
        Ok(Wordlist(
            parts[0].to_string(),
            parts[1].split(',').map(|x| x.to_string()).collect(),
        ))
    } else {
        Err("Invalid wordlist".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        assert_eq!(
            parse_url("http://example.com").unwrap(),
            "http://example.com/".to_string()
        );
        assert_eq!(
            parse_url("https://example.com").unwrap(),
            "https://example.com/".to_string()
        );
        assert_eq!(
            parse_url("example.com").unwrap(),
            "http://example.com/".to_string()
        );
        assert_eq!(
            parse_url("http://example.com:8080").unwrap(),
            "http://example.com:8080/".to_string()
        );
        assert_eq!(
            parse_url("http://example.com:8080/path").unwrap(),
            "http://example.com:8080/path".to_string()
        );
        assert_eq!(
            parse_url("http://example.com:8080/path?query").unwrap(),
            "http://example.com:8080/path?query".to_string()
        );
        assert_eq!(
            parse_url("http://example.com:8080/path?query#fragment").unwrap(),
            "http://example.com:8080/path?query#fragment".to_string()
        );
        assert!(parse_url("http://").is_err());
        assert!(parse_url("http://example.com:8080:").is_err());
        assert!(parse_url("http://example.com:8080:8080").is_err());
        assert!(parse_url("http://example.com:8080:8080/path").is_err());
        assert!(parse_url("http://example.com:8080:8080/path?query").is_err());
        assert!(parse_url("http://example.com:8080:8080/path?query#fragment").is_err());
    }

    #[test]
    fn test_parse_header() {
        assert_eq!(parse_header("key:value").unwrap(), "key:value".to_string());
        assert_eq!(parse_header("key:").unwrap(), "key:".to_string());
        assert_eq!(parse_header(":value").unwrap(), ":value".to_string());
        assert!(parse_header("key").is_err());
    }

    #[test]
    fn test_parse_cookie() {
        assert_eq!(parse_cookie("key=value").unwrap(), "key=value".to_string());
        assert!(parse_cookie("key").is_err());
        assert_eq!(parse_cookie("=value").unwrap(), "=value".to_string());
        assert_eq!(parse_cookie("key=").unwrap(), "key=".to_string());
    }

    #[test]
    fn test_parse_method() {
        assert_eq!(parse_method("GET").unwrap(), "GET".to_string());
        assert_eq!(parse_method("get").unwrap(), "GET".to_string());
        assert_eq!(parse_method("POST").unwrap(), "POST".to_string());
        assert_eq!(parse_method("post").unwrap(), "POST".to_string());
        assert_eq!(parse_method("PUT").unwrap(), "PUT".to_string());
        assert_eq!(parse_method("put").unwrap(), "PUT".to_string());
        assert_eq!(parse_method("DELETE").unwrap(), "DELETE".to_string());
        assert_eq!(parse_method("delete").unwrap(), "DELETE".to_string());
        assert_eq!(parse_method("HEAD").unwrap(), "HEAD".to_string());
        assert_eq!(parse_method("head").unwrap(), "HEAD".to_string());
        assert_eq!(parse_method("OPTIONS").unwrap(), "OPTIONS".to_string());
        assert_eq!(parse_method("options").unwrap(), "OPTIONS".to_string());
        assert_eq!(parse_method("TRACE").unwrap(), "TRACE".to_string());
        assert_eq!(parse_method("trace").unwrap(), "TRACE".to_string());
        assert_eq!(parse_method("CONNECT").unwrap(), "CONNECT".to_string());
        assert_eq!(parse_method("connect").unwrap(), "CONNECT".to_string());
        assert!(parse_method("INVALID").is_err());
    }

    #[test]
    fn test_parse_wordlist() {
        assert_eq!(
            parse_wordlist("wordlist").unwrap(),
            Wordlist("wordlist".to_string(), vec![])
        );
        assert_eq!(
            parse_wordlist("key:wordlist").unwrap(),
            Wordlist("key".to_string(), vec!["wordlist".to_string()])
        );
        assert_eq!(
            parse_wordlist("key:wordlist1,wordlist2").unwrap(),
            Wordlist(
                "key".to_string(),
                vec!["wordlist1".to_string(), "wordlist2".to_string()]
            )
        );
        assert_eq!(
            parse_wordlist("key:").unwrap(),
            Wordlist("key".to_string(), vec!["".to_string()])
        );
        assert!(parse_wordlist("key:wordlist1,wordlist2:").is_err());
    }
}
