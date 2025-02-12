pub mod contains;
pub mod ends;
pub mod length;
pub mod starts;

use crate::{
    filters::{expression::FilterExpr, Filter},
    utils::registry::create_registry,
    Result,
};

use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

create_registry!(
    filter,
    WordlistFilterRegitry,
    String,
    [
        length::LengthFilter,
        contains::ContainsFilter,
        starts::StartsFilter,
        ends::EndsFilter
    ]
);
