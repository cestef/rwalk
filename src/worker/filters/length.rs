use super::response_filter;
use crate::utils::types::IntRange;
use crate::{error, RwalkError};

response_filter!(
    LengthFilter,
    Vec<IntRange<usize>>,
    needs_body = true,
    |res: &RwalkResponse, range: &Vec<IntRange<usize>>| {
        let body = res.body.as_ref().ok_or_else(|| error!("No body found"))?;
        Ok(range.iter().any(|r| r.contains(body.len())))
    },
    "length",
    ["l", "size"],
    transform = |raw: String| raw.split(',').map(|s| s.parse()).collect::<Result<_>>()
);
