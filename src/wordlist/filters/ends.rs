use super::wordlist_filter;

wordlist_filter!(
    EndsFilter,
    String,
    |w: &String, sub: &String| w.ends_with(sub),
    "ends",
    "end",
    "e"
);
