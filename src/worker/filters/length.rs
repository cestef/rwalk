use crate::utils::types::IntRange;

use super::response_filter;

response_filter!(
    LengthFilter,
    IntRange<usize>,
    |res, range| res.body.as_ref().map_or(false, |e| range.contains(e.len())),
    "length",
    ["l", "size"],
    transform = |raw: String| raw.parse()
);
