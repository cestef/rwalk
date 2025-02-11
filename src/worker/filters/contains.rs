use super::response_filter;

response_filter!(
    ContainsFilter,
    String,
    |res, sub| res.body.as_ref().map_or(false, |e| e.contains(sub)),
    "contains",
    "c"
);
