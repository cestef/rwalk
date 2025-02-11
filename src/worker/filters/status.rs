use crate::utils::types::IntRange;

use super::response_filter;

response_filter!(
    StatusFilter,
    Vec<IntRange<u16>>,
    needs_body = false,
    |res: &RwalkResponse, range: &Vec<IntRange<u16>>| range.iter().any(|r| r.contains(res.status)),
    "status",
    ["code", "s"],
    transform = |raw: String| raw.split(',').map(|s| s.parse()).collect::<Result<_>>()
);
