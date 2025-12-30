use super::wordlist_filter;

use regex::Regex;

wordlist_filter!(
    RegexFilter,
    Regex,
    |w: &CowStr, re: &Regex| Ok(re.is_match(w)),
    "regex",
    ["r"],
    transform = |raw: String| Regex::new(&raw)
);
