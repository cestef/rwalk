use super::response_filter;
use crate::{error, RwalkError};

response_filter!(
    TypeFilter,
    bool,
    needs_body = false,
    |res: &RwalkResponse, needs_dir: &bool| res.directory == *needs_dir,
    "type",
    ["t"],
    transform = |raw: String| match raw.to_lowercase().as_str() {
        "directory" | "dir" | "d" => Ok(true),
        "file" | "f" => Ok(false),
        _ => Err(error!("Invalid type filter value: {}", raw)),
    }
);
