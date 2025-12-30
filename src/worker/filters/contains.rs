use super::response_filter;

response_filter!(
    ContainsFilter,
    String,
    needs_body = true,
    |res: &RwalkResponse, sub: &String| Ok(res.body.contains(sub)),
    "contains",
    "c"
);
