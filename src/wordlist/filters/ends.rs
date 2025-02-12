use super::wordlist_filter;

wordlist_filter!(
    EndsFilter,
    String,
    |w: &CowStr, sub: &String| w.ends_with(sub),
    "ends",
    "end",
    "e"
);
