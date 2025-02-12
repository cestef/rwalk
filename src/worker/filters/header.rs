use super::response_filter;

use crate::cli::parse::parse_keyval_with_sep;

response_filter!(
    HeaderFilter,
    (String, String),
    needs_body = false,
    |res: &RwalkResponse, keyval: &(String, String)| res
        .headers
        .get(&keyval.0)
        .map_or(false, |r| r.value() == &keyval.1),
    "header",
    ["h"],
    transform = |raw: String| parse_keyval_with_sep(&raw, '=')
);
