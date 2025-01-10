pub mod contains;
pub mod ends;
pub mod length;
pub mod starts;
pub mod status;

use crate::{
    error::RwalkError,
    filters::{create_filter_registry, Filter},
    worker::utils::RwalkResponse,
    Result,
};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

create_filter_registry!(
    ResponseFilterRegistry,
    RwalkResponse,
    [
        status::StatusFilter,
        length::LengthFilter,
        starts::StartsFilter,
        ends::EndsFilter,
        contains::ContainsFilter
    ]
);
