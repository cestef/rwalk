pub mod contains;
pub mod ends;
pub mod length;
pub mod starts;

use crate::{
    error::RwalkError,
    filters::{create_filter_registry, Filter},
    Result,
};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

create_filter_registry!(
    WordlistFilterRegitry,
    String,
    [
        length::LengthFilter,
        contains::ContainsFilter,
        starts::StartsFilter,
        ends::EndsFilter
    ]
);
