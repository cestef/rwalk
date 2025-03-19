use super::response_filter;

response_filter!(
    StartsFilter,
    String,
    needs_body = true,
    |res: &RwalkResponse, sub: &String| Ok(res.body.as_ref().map_or(false, |e| e.starts_with(sub))),
    "starts",
    "begin"
);
