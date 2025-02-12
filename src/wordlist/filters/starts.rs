use super::wordlist_filter;

wordlist_filter!(
    StartsFilter,
    String,
    |w: &String, sub: &String| w.starts_with(sub),
    "starts",
    "start",
    "e"
);
