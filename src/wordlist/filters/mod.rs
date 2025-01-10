pub mod length;

use crate::{
    error::RwalkError,
    filters::{create_filter_registry, Filter},
    Result,
};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

create_filter_registry!(WORDLIST_FILTER_REGISTRY, String, [length::LengthFilter]);

pub struct WordlistFilterRegistry;

impl WordlistFilterRegistry {
    pub fn construct(name: &str, arg: &str) -> Result<Box<dyn Filter<String>>> {
        match WORDLIST_FILTER_REGISTRY.get(name) {
            Some(constructor) => constructor(arg),
            None => Err(crate::error!("Unknown filter: {}", name)),
        }
    }

    pub fn list() -> HashSet<&'static str> {
        WORDLIST_FILTER_REGISTRY.keys().copied().collect()
    }
}
