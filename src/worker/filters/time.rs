use crate::utils::types::IntRange;

use super::response_filter;

response_filter!(
    TimeFilter,
    Vec<IntRange<u64>>,
    needs_body = false,
    |res: &RwalkResponse, range: &Vec<IntRange<u64>>| Ok(range
        .iter()
        .any(|r| r.contains(res.time.as_secs()))),
    "time",
    ["elapsed", "t"],
    transform = |raw: String| raw.split(',').map(|s| s.parse()).collect::<Result<_>>()
);
