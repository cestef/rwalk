use super::response_filter;

response_filter!(
    EndsFilter,
    String,
    needs_body = true,
    |res: &RwalkResponse, sub: &String| Ok(res.body.ends_with(sub)),
    "ends",
    "e"
);
