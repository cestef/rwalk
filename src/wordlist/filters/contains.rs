use super::wordlist_filter;

wordlist_filter!(
    ContainsFilter,
    String,
    |w: &CowStr, sub: &String| Ok(w.contains(sub)),
    "contains",
    "c"
);
