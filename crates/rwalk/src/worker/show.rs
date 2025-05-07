use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::utils::format::display_time;

use super::utils::RwalkResponse;

fn show_body(res: &RwalkResponse) -> String {
    res.body.clone()
}

fn show_type(res: &RwalkResponse) -> String {
    res.r#type.to_string()
}

// k1:v1, k2:v2
fn show_headers(res: &RwalkResponse) -> String {
    res.headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<_>>()
        .join(", ")
}

fn show_time(res: &RwalkResponse) -> String {
    display_time(res.time)
}

fn show_status(res: &RwalkResponse) -> String {
    res.status.to_string()
}

fn show_length(res: &RwalkResponse) -> String {
    res.body.len().to_string()
}

fn boxed_fn(
    f: fn(&RwalkResponse) -> String,
) -> Box<dyn Fn(&RwalkResponse) -> String + Send + Sync> {
    Box::new(f)
}

lazy_static! {
    pub static ref SHOW_REGISTRY: HashMap<String, Box<dyn Fn(&RwalkResponse) -> String + Send + Sync>> = {
        let mut m = HashMap::new();
        m.insert("body".to_string(), boxed_fn(show_body));
        m.insert("headers".to_string(), boxed_fn(show_headers));
        m.insert("type".to_string(), boxed_fn(show_type));
        m.insert("time".to_string(), boxed_fn(show_time));
        m.insert("status".to_string(), boxed_fn(show_status));
        m.insert("length".to_string(), boxed_fn(show_length));
        m
    };
}
