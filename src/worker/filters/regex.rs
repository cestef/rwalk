use super::response_filter;

use regex::Regex;

response_filter!(
    RegexFilter,
    Regex,
    needs_body = true,
    |res: &RwalkResponse, re: &Regex| Ok(re.is_match(&res.body)),
    "regex",
    ["r"],
    transform = |raw: String| Regex::new(&raw)
);
