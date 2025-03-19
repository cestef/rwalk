pub mod contains;
pub mod ends;
pub mod length;
pub mod starts;

use crate::{
    filters::{expression::FilterExpr, Filter},
    utils::registry::create_registry,
    Result,
};

use cowstr::CowStr;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

create_registry!(
    filter,
    WordlistFilterRegistry,
    (CowStr, CowStr),
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
        use cowstr::CowStr;
        use super::Filter;
        use crate::{
            Result,
            filters::evaluator::GenericEvaluator,
        };
        use std::collections::HashSet;

        static EVALUATOR: Lazy<GenericEvaluator<$value_type, CowStr>> = Lazy::new(|| {
            GenericEvaluator::new($filter_fn)
        });

        #[derive(Debug, Clone)]
        pub struct $filter_name {
            value: $value_type,
            filter: Option<HashSet<CowStr>>,
        }

        impl Filter<(CowStr, CowStr)> for $filter_name {
            fn filter(&self, item: &(CowStr, CowStr)) -> Result<bool> {
                if let Some(filter) = &self.filter {
                    if !filter.contains(&item.0) {
                        return Ok(true);
                    }
                }
                $filter_fn(&item.1, &self.value)
            }

            fn name() -> &'static str {
                $filter_str
            }

            fn aliases() -> &'static [&'static str] {
                &[$($alias),*]
            }

            fn construct(arg: &str, filter: Option<HashSet<CowStr>>) -> Result<Box<dyn Filter<(CowStr, CowStr)>>>
            where
                Self: Sized,
            {
                let value = $transform(arg.to_string())?;
                Ok(Box::new(Self { value, filter }))
            }
        }

        impl std::fmt::Display for $filter_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", Self::name(), self.value)
            }
        }
    };
}

pub(crate) use wordlist_filter;
