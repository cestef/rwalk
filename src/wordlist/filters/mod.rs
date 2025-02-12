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

macro_rules! wordlist_filter {
    // Default transformation
    (
        $filter_name:ident,
        $value_type:ty,
        $filter_fn:expr,
        $filter_str:literal,
        $($alias:literal),* $(,)?
    ) => {
        wordlist_filter!(
            $filter_name,
            $value_type,
            $filter_fn,
            $filter_str,
            [$($alias),*],
            transform = |raw: String| -> Result<$value_type> { Ok(raw) }
        );
    };


    (
        $filter_name:ident,
        $value_type:ty,
        $filter_fn:expr,
        $filter_str:literal,
        [$($alias:literal),* $(,)?],
        transform = $transform:expr
    ) => {
        use once_cell::sync::Lazy;
        use super::Filter;
        use crate::{
            Result,
            filters::evaluator::GenericEvaluator,
        };

        static EVALUATOR: Lazy<GenericEvaluator<$value_type, String>> = Lazy::new(|| {
            GenericEvaluator::new($filter_fn)
        });

        #[derive(Debug, Clone)]
        pub struct $filter_name {
            value: $value_type,
        }

        impl Filter<String> for $filter_name {
            fn filter(&self, item: &String) -> bool {
                $filter_fn(item, &self.value)
            }

            fn name() -> &'static str {
                $filter_str
            }

            fn aliases() -> &'static [&'static str] {
                &[$($alias),*]
            }

            fn construct(arg: &str, _specifier: Option<&str>) -> Result<Box<dyn Filter<String>>>
            where
                Self: Sized,
            {
                let value = $transform(arg.to_string())?;
                Ok(Box::new(Self { value }))
            }
        }
    };
}

pub(crate) use wordlist_filter;
