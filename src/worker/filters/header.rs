use smartstring::{LazyCompact, SmartString};

use super::response_filter;

use crate::cli::parse::parse_keyval_with_sep;

response_filter!(
    HeaderFilter,
    (SmartString<LazyCompact>, SmartString<LazyCompact>),
    needs_body = false,
    |res: &RwalkResponse, keyval: &(SmartString<LazyCompact>, SmartString<LazyCompact>)| Ok(res
        .headers
        .get(&keyval.0)
        .is_some_and(|e| *e.as_immutable_string_ref().unwrap()
            == keyval.1)),
    "header",
    ["h"],
    transform = |raw: String| parse_keyval_with_sep(&raw, '=').map(|(k, v)| (k.into(), v.into()))
);
