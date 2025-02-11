use super::response_filter;

response_filter!(
    ContainsFilter,
    String,
    needs_body = true,
    |res, sub| res.body.as_ref().map_or(false, |e| e.contains(sub)),
    "contains",
    "c"
);
