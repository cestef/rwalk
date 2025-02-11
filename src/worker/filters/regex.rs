use super::response_filter;

use regex::Regex;

response_filter!(
    RegexFilter,
    Regex,
    needs_body = true,
    |res: &RwalkResponse, re: &Regex| res.body.as_ref().map_or(false, |e| re.is_match(e)),
    "regex",
    ["r"],
    transform = |raw: String| Regex::new(&raw)
);
