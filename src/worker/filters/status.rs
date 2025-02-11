use crate::utils::types::IntRange;

use super::response_filter;

response_filter!(
    StatusFilter,
    IntRange<u16>,
    |res, range| range.contains(res.status.as_u16()),
    "status",
    ["code", "s"],
    transform = |raw: String| raw.parse()
);
