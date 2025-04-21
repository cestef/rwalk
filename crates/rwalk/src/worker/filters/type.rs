use super::response_filter;
use crate::worker::utils::ResponseType;

response_filter!(
    TypeFilter,
    ResponseType,
    needs_body = false,
    |res: &RwalkResponse, expected: &ResponseType| Ok(res.r#type == *expected),
    "type",
    ["t"],
    transform = |raw: String| raw.parse::<ResponseType>()
);
