pub mod contains;
pub mod ends;
pub mod length;
pub mod starts;
pub mod status;
pub mod time;

use crate::{
    filters::{
        create_filter_registry,
        expression::{Evaluator, FilterExpr},
        Filter,
    },
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
        contains::ContainsFilter,
        time::TimeFilter
    ]
);

pub struct GenericResponseEvaluator<V> {
    filter_fn: fn(&RwalkResponse, &V) -> bool,
    _phantom: std::marker::PhantomData<V>,
}

impl<V> GenericResponseEvaluator<V> {
    pub fn new(filter_fn: fn(&RwalkResponse, &V) -> bool) -> Self {
        Self {
            filter_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<V> Evaluator<RwalkResponse, V> for GenericResponseEvaluator<V> {
    fn evaluate(&self, expr: &FilterExpr<V>, item: &RwalkResponse) -> bool {
        use FilterExpr::*;
        match expr {
            And(left, right) => self.evaluate(left, item) && self.evaluate(right, item),
            Or(left, right) => self.evaluate(left, item) || self.evaluate(right, item),
            Not(expr) => !self.evaluate(expr, item),
            Value(sub) => (self.filter_fn)(item, sub),
            Raw(e) => unreachable!("{e}"), // Should not happen after parsing
        }
    }
}
#[macro_export]
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

    // Variant with custom value transformation
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
        use super::{Filter, GenericResponseEvaluator};
        use crate::{
            worker::utils::RwalkResponse,
            Result,
        };

        static EVALUATOR: Lazy<GenericResponseEvaluator<$value_type>> = Lazy::new(|| {
            GenericResponseEvaluator::new($filter_fn)
        });

        #[derive(Debug, Clone)]
        pub struct $filter_name {
            value: $value_type,
        }

        impl Filter<RwalkResponse> for $filter_name {
            fn filter(&self, item: &RwalkResponse) -> bool {
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

            fn construct(arg: &str) -> Result<Box<dyn Filter<RwalkResponse>>>
            where
                Self: Sized,
            {
                let value = $transform(arg.to_string())?;

                Ok(Box::new(Self { value }))
            }
        }
    };
}

pub(crate) use response_filter;
