use super::response_filter;

response_filter!(
    EndsFilter,
    String,
    |res, sub| res.body.as_ref().map_or(false, |e| e.ends_with(sub)),
    "ends",
    "e"
);
