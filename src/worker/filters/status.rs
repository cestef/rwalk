use crate::utils::types::IntRange;

use super::response_filter;

response_filter!(
    StatusFilter,
    IntRange<u16>,
    needs_body = false,
    |res, range| range.contains(res.status),
    "status",
    ["code", "s"],
    transform = |raw: String| raw.parse()
);
