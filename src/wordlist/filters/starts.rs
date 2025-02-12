use super::wordlist_filter;

wordlist_filter!(
    StartsFilter,
    String,
    |w: &CowStr, sub: &String| w.starts_with(sub),
    "starts",
    "start",
    "e"
);
