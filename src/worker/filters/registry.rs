use crate::{error::RwalkError, Result};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

use crate::worker::{
    filters::{length::LengthFilter, status::StatusFilter, Filter},
    utils::SendableResponse,
};

type FilterConstructor = fn(&str) -> Result<Box<dyn Filter<SendableResponse>>>;
static FILTER_REGISTRY: Lazy<HashMap<&'static str, FilterConstructor>> = Lazy::new(|| {
    let mut registry = HashMap::new();

    macro_rules! register_filter {
        ($filter:ty) => {
            // Register main name
            registry.insert(<$filter>::name(), <$filter>::construct as FilterConstructor);
            // Register aliases
            for &alias in <$filter>::aliases() {
                registry.insert(alias, <$filter>::construct as FilterConstructor);
            }
        };
    }

    register_filter!(StatusFilter);
    register_filter!(LengthFilter);

    registry
});

pub struct DefaultFilterRegistry;

impl DefaultFilterRegistry {
    pub fn construct(name: &str, arg: &str) -> Result<Box<dyn Filter<SendableResponse>>> {
        match FILTER_REGISTRY.get(name) {
            Some(constructor) => constructor(arg),
            None => Err(crate::error!("Unknown filter: {}", name)),
        }
    }

    pub fn list() -> HashSet<&'static str> {
        FILTER_REGISTRY.keys().copied().collect()
    }
}
