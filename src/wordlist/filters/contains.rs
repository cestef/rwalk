use super::wordlist_filter;

wordlist_filter!(
    ContainsFilter,
    String,
    |w: &String, sub: &String| w.contains(sub),
    "contains",
    "c"
);
