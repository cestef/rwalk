use super::response_filter;

response_filter!(
    EndsFilter,
    String,
    needs_body = true,
    |res, sub| res.body.as_ref().map_or(false, |e| e.ends_with(sub)),
    "ends",
    "e"
);
