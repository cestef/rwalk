//! Proxy types for facet opaque field support
//!
//! This module provides proxy types that handle transformations for fields
//! that require custom serialization/deserialization logic.

use dashmap::DashSet;
use facet::Facet;
use url::Url;

use crate::utils::constants::DEFAULT_WORDLIST_KEY;

//
// ──────────────────────────────────────────────────────────────────
// URL Proxy
// ──────────────────────────────────────────────────────────────────
//

/// Proxy type for URL that auto-adds http:// prefix when missing
#[derive(Debug, Clone, Facet, serde::Serialize, serde::Deserialize)]
#[facet(transparent)]
pub struct UrlProxy(String);

impl From<&Option<Url>> for UrlProxy {
    fn from(url: &Option<Url>) -> Self {
        match url {
            Some(u) => UrlProxy(u.to_string()),
            None => UrlProxy(String::new()),
        }
    }
}

impl TryFrom<UrlProxy> for Option<Url> {
    type Error = String;

    fn try_from(proxy: UrlProxy) -> Result<Self, Self::Error> {
        if proxy.0.is_empty() {
            return Ok(None);
        }

        // Add http:// prefix if not present
        let url_str = if !proxy.0.starts_with("http://") && !proxy.0.starts_with("https://") {
            format!("http://{}", proxy.0)
        } else {
            proxy.0
        };

        Url::parse(&url_str)
            .map(Some)
            .map_err(|e| format!("Invalid URL: {}", e))
    }
}

//
// ──────────────────────────────────────────────────────────────────
// Wordlist Proxy
// ──────────────────────────────────────────────────────────────────
//

/// Proxy type for wordlists that handles "path:key" notation
///
/// Converts between Vec<(String, String)> and Vec<String> where each string
/// can be either "path" (uses default key) or "path:key" format.
#[derive(Debug, Clone, Facet, serde::Serialize, serde::Deserialize)]
#[facet(transparent)]
pub struct WordlistProxy(Vec<String>);

impl From<&Vec<(String, String)>> for WordlistProxy {
    fn from(wordlists: &Vec<(String, String)>) -> Self {
        let strings = wordlists
            .iter()
            .map(|(path, key)| {
                if key == DEFAULT_WORDLIST_KEY {
                    path.clone()
                } else {
                    format!("{}:{}", path, key)
                }
            })
            .collect();
        WordlistProxy(strings)
    }
}

impl TryFrom<WordlistProxy> for Vec<(String, String)> {
    type Error = String;

    fn try_from(proxy: WordlistProxy) -> Result<Self, Self::Error> {
        let mut result = Vec::new();

        for item in proxy.0 {
            let parts: Vec<&str> = item.split(':').collect();
            let (path, key) = if parts.len() >= 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                (item, DEFAULT_WORDLIST_KEY.to_string())
            };
            result.push((path, key));
        }

        Ok(result)
    }
}

//
// ──────────────────────────────────────────────────────────────────
// IntRange Proxy
// ──────────────────────────────────────────────────────────────────
//

use crate::utils::types::IntRange;

/// Proxy for Vec<IntRange<u16>> used by retry_codes field
#[derive(Debug, Clone, Facet, serde::Serialize, serde::Deserialize)]
#[facet(transparent)]
pub struct IntRangeU16VecProxy(Vec<(u16, u16)>);

impl From<&Vec<IntRange<u16>>> for IntRangeU16VecProxy {
    fn from(ranges: &Vec<IntRange<u16>>) -> Self {
        let tuples = ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect();
        IntRangeU16VecProxy(tuples)
    }
}

impl TryFrom<IntRangeU16VecProxy> for Vec<IntRange<u16>> {
    type Error = String;

    fn try_from(proxy: IntRangeU16VecProxy) -> Result<Self, Self::Error> {
        let ranges = proxy
            .0
            .into_iter()
            .map(|(start, end)| IntRange::new(start, end))
            .collect();
        Ok(ranges)
    }
}

//
// ──────────────────────────────────────────────────────────────────
// Complex Tuple Proxies for DashSet-containing fields
// ──────────────────────────────────────────────────────────────────
//

/// Proxy for Vec<(HashSet<String>, String, Option<String>)> used by transforms field
#[derive(Debug, Clone, Facet, serde::Serialize, serde::Deserialize)]
#[facet(transparent)]
pub struct TransformsProxy(Vec<(Vec<String>, String, Option<String>)>);

impl From<&Vec<(DashSet<String>, String, Option<String>)>> for TransformsProxy {
    fn from(transforms: &Vec<(DashSet<String>, String, Option<String>)>) -> Self {
        let vec = transforms
            .iter()
            .map(|(set, s1, opt)| {
                let vec: Vec<String> = set.iter().map(|item| item.clone()).collect();
                (vec, s1.clone(), opt.clone())
            })
            .collect();
        TransformsProxy(vec)
    }
}

impl TryFrom<TransformsProxy> for Vec<(DashSet<String>, String, Option<String>)> {
    type Error = String;

    fn try_from(proxy: TransformsProxy) -> Result<Self, Self::Error> {
        let result = proxy
            .0
            .into_iter()
            .map(|(vec, s1, opt)| {
                let set = DashSet::new();
                for item in vec {
                    set.insert(item);
                }
                (set, s1, opt)
            })
            .collect();
        Ok(result)
    }
}

/// Proxy for Vec<(HashSet<String>, String)> used by merge field
#[derive(Debug, Clone, Facet, serde::Serialize, serde::Deserialize)]
#[facet(transparent)]
pub struct MergeProxy(Vec<(Vec<String>, String)>);

impl From<&Vec<(DashSet<String>, String)>> for MergeProxy {
    fn from(merges: &Vec<(DashSet<String>, String)>) -> Self {
        let vec = merges
            .iter()
            .map(|(set, s)| {
                let vec: Vec<String> = set.iter().map(|item| item.clone()).collect();
                (vec, s.clone())
            })
            .collect();
        MergeProxy(vec)
    }
}

impl TryFrom<MergeProxy> for Vec<(DashSet<String>, String)> {
    type Error = String;

    fn try_from(proxy: MergeProxy) -> Result<Self, Self::Error> {
        let result = proxy
            .0
            .into_iter()
            .map(|(vec, s)| {
                let set = DashSet::new();
                for item in vec {
                    set.insert(item);
                }
                (set, s)
            })
            .collect();
        Ok(result)
    }
}

/// Proxy for Vec<(HashSet<String>, String, String)> used by headers field
#[derive(Debug, Clone, Facet, serde::Serialize, serde::Deserialize)]
#[facet(transparent)]
pub struct HeadersProxy(Vec<(Vec<String>, String, String)>);

impl From<&Vec<(DashSet<String>, String, String)>> for HeadersProxy {
    fn from(headers: &Vec<(DashSet<String>, String, String)>) -> Self {
        let vec = headers
            .iter()
            .map(|(set, s1, s2)| {
                let vec: Vec<String> = set.iter().map(|item| item.clone()).collect();
                (vec, s1.clone(), s2.clone())
            })
            .collect();
        HeadersProxy(vec)
    }
}

impl TryFrom<HeadersProxy> for Vec<(DashSet<String>, String, String)> {
    type Error = String;

    fn try_from(proxy: HeadersProxy) -> Result<Self, Self::Error> {
        let result = proxy
            .0
            .into_iter()
            .map(|(vec, s1, s2)| {
                let set = DashSet::new();
                for item in vec {
                    set.insert(item);
                }
                (set, s1, s2)
            })
            .collect();
        Ok(result)
    }
}
