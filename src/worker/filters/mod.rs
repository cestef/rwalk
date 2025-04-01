mod contains;
mod ends;
mod header;
mod length;
mod regex;
mod script;
mod starts;
mod status;
mod time;
mod r#type;

use crate::{
    filters::{expression::FilterExpr, Filter},
    utils::registry::create_registry,
    worker::utils::RwalkResponse,
    Result,
};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

create_registry!(
    filter,
    ResponseFilterRegistry,
    RwalkResponse,
    [
        status::StatusFilter,
        length::LengthFilter,
        starts::StartsFilter,
        ends::EndsFilter,
        contains::ContainsFilter,
        time::TimeFilter,
        regex::RegexFilter,
        header::HeaderFilter,
        r#type::TypeFilter,
        script::ScriptFilter,
    ]
);

macro_rules! response_filter {
    // Basic variant with default transformation
    (
        $filter_name:ident,
        $value_type:ty,
        needs_body = $needs_body:expr,
        $filter_fn:expr,
        $filter_str:literal,
        $($alias:literal),* $(,)?
    ) => {
        response_filter!(
            $filter_name,
            $value_type,
            needs_body = $needs_body,
            $filter_fn,
            $filter_str,
            [$($alias),*],
            transform = |raw: String| -> Result<$value_type> { Ok(raw) }
        );
    };


    (
        $filter_name:ident,
        $value_type:ty,
        needs_body = $needs_body:expr,
        $filter_fn:expr,
        $filter_str:literal,
        [$($alias:literal),* $(,)?],
        transform = $transform:expr
    ) => {
        use once_cell::sync::Lazy;
        use super::Filter;
        use crate::{
            worker::utils::RwalkResponse,
            Result,
            filters::evaluator::GenericEvaluator,
        };
        use std::collections::HashSet;
        use cowstr::CowStr;

        static EVALUATOR: Lazy<GenericEvaluator<$value_type, RwalkResponse>> = Lazy::new(|| {
            GenericEvaluator::new($filter_fn)
        });

        #[derive(Debug, Clone)]
        pub struct $filter_name {
            value: $value_type,
            depth: Option<HashSet<usize>>,
        }

        impl Filter<RwalkResponse> for $filter_name {
            fn filter(&self, item: &RwalkResponse) -> Result<bool> {
                if let Some(ref depth) = self.depth {
                    if !depth.contains(&(item.depth as usize)) {
                        // Skip if the depth does not match
                        return Ok(true);
                    }
                }
                $filter_fn(item, &self.value)
            }

            fn needs_body(&self) -> bool {
                $needs_body
            }

            fn name() -> &'static str {
                $filter_str
            }

            fn aliases() -> &'static [&'static str] {
                &[$($alias),*]
            }

            fn construct(arg: &str, depth: Option<HashSet<CowStr>>) -> Result<Box<dyn Filter<RwalkResponse>>>
            where
                Self: Sized,
            {
                let value = $transform(arg.to_string())?;
                let depth = if let Some(depth) = depth {
                    Some(depth.iter().map(|d| d.parse().unwrap()).collect())
                } else {
                    None
                };
                Ok(Box::new(Self { value, depth }))
            }
        }

        impl std::fmt::Display for $filter_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use owo_colors::OwoColorize;
                use itertools::Itertools;
                write!(f, "{}{}:{:?}",
                    if let Some(ref depth) = self.depth {
                        format!("[{}]", depth.iter().sorted().map(|d| d.dimmed().to_string()).collect::<Vec<String>>().join(","))
                    } else {
                        "".to_string()
                    },
                    Self::name(),
                    self.value
                )
            }
        }
    };
}

pub(crate) use response_filter;
