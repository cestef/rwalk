use super::response_filter;

response_filter!(
    StartsFilter,
    String,
    needs_body = true,
    |res: &RwalkResponse, sub: &String| Ok(res.body.starts_with(sub)),
    "starts",
    "begin"
);
