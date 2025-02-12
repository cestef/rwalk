use super::wordlist_filter;

wordlist_filter!(
    LengthFilter,
    usize,
    |w: &String, len: &usize| w.len() == *len,
    "starts",
    ["start", "e"],
    transform = |raw: String| -> Result<String> { Ok(raw.parse::<usize>().map(|_| raw)?) }
);
