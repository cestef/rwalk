use super::wordlist_filter;

wordlist_filter!(
    LengthFilter,
    usize,
    |w: &CowStr, len: &usize| Ok(w.len() == *len),
    "starts",
    ["start", "e"],
    transform = |raw: String| -> Result<usize> { Ok(raw.parse::<usize>()?) }
);
