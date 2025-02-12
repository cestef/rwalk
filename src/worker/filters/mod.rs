mod contains;
mod ends;
mod header;
mod length;
mod regex;
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
        r#type::TypeFilter
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

        static EVALUATOR: Lazy<GenericEvaluator<$value_type, RwalkResponse>> = Lazy::new(|| {
            GenericEvaluator::new($filter_fn)
        });

        #[derive(Debug, Clone)]
        pub struct $filter_name {
            value: $value_type,
            depth: Option<usize>,
        }

        impl Filter<RwalkResponse> for $filter_name {
            fn filter(&self, item: &RwalkResponse) -> bool {
                if let Some(depth) = self.depth {
                    if item.depth != depth {
                        return false;
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

            fn construct(arg: &str, depth: Option<&str>) -> Result<Box<dyn Filter<RwalkResponse>>>
            where
                Self: Sized,
            {
                let value = $transform(arg.to_string())?;
                let depth = if let Some(depth) = depth {
                    Some(depth.parse::<usize>()?)
                } else {
                    None
                };
                Ok(Box::new(Self { value, depth }))
            }
        }
    };
}

pub(crate) use response_filter;
