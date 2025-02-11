use crate::utils::types::IntRange;

use super::response_filter;

response_filter!(
    LengthFilter,
    Vec<IntRange<usize>>,
    needs_body = true,
    |res: &RwalkResponse, range: &Vec<IntRange<usize>>| {
        let body = res.body.as_ref().expect("body is needed for length filter");
        range.iter().any(|r| r.contains(body.len()))
    },
    "length",
    ["l", "size"],
    transform = |raw: String| raw.split(',').map(|s| s.parse()).collect::<Result<_>>()
);
