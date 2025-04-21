use super::response_filter;
use crate::utils::types::IntRange;

response_filter!(
    LengthFilter,
    Vec<IntRange<usize>>,
    needs_body = true,
    |res: &RwalkResponse, range: &Vec<IntRange<usize>>| {
        Ok(range.iter().any(|r| r.contains(res.body.len())))
    },
    "length",
    ["l", "size"],
    transform = |raw: String| raw.split(',').map(|s| s.parse()).collect::<Result<_>>()
);
