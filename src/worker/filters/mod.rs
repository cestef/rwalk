pub mod length;
pub mod status;

use crate::{
    error::RwalkError,
    filters::{create_filter_registry, Filter},
    worker::utils::SendableResponse,
    Result,
};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

create_filter_registry!(
    RESPONSE_FILTER_REGISTRY,
    SendableResponse,
    [status::StatusFilter, length::LengthFilter]
);

pub struct ResponseFilterRegistry;

impl ResponseFilterRegistry {
    pub fn construct(name: &str, arg: &str) -> Result<Box<dyn Filter<SendableResponse>>> {
        match RESPONSE_FILTER_REGISTRY.get(name) {
            Some(constructor) => constructor(arg),
            None => Err(crate::error!("Unknown filter: {}", name)),
        }
    }

    pub fn list() -> HashSet<&'static str> {
        RESPONSE_FILTER_REGISTRY.keys().copied().collect()
    }
}
