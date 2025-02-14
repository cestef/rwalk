use crate::Result;
use cowstr::CowStr;
use evaluator::Evaluator;
use expression::FilterExpr;
use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    sync::Arc,
};

pub mod evaluator;
pub mod expression;

#[derive(Debug, Clone)]
pub struct Filterer<T> {
    pub filter: Option<Arc<FilterExpr<Box<dyn Filter<T>>>>>,
}

unsafe impl<T> Send for Filterer<T> where Box<dyn Filter<T>>: Send {}
unsafe impl<T> Sync for Filterer<T> where Box<dyn Filter<T>>: Sync {}

pub trait Filter<T>: Debug + Send + Sync + Display {
    fn filter(&self, item: &T) -> bool;
    fn name() -> &'static str
    where
        Self: Sized;

    fn aliases() -> &'static [&'static str]
    where
        Self: Sized,
    {
        &[]
    }
    fn needs_body(&self) -> bool {
        false
    }
    fn construct(arg: &str, specifier: Option<HashSet<CowStr>>) -> Result<Box<dyn Filter<T>>>
    where
        Self: Sized;
}

impl<T> Filterer<T> {
    pub fn new(filter: Option<FilterExpr<Box<dyn Filter<T>>>>) -> Self {
        Self {
            filter: filter.map(Arc::new),
        }
    }

    pub fn filter(&self, item: &T) -> bool {
        if let Some(filter) = &self.filter {
            FILTER_EVALUATOR.evaluate(filter, item)
        } else {
            true
        }
    }

    pub fn needs_body(&self) -> bool {
        if let Some(filter) = &self.filter {
            NEEDS_BODY_EVALUATOR.evaluate(filter, &())
        } else {
            false
        }
    }
}

#[derive(Debug)]
struct FilterEvaluator;

impl<T> Evaluator<T, Box<dyn Filter<T>>> for FilterEvaluator {
    fn evaluate(&self, expr: &FilterExpr<Box<dyn Filter<T>>>, item: &T) -> bool {
        match expr {
            FilterExpr::And(left, right) => self.evaluate(left, item) && self.evaluate(right, item),
            FilterExpr::Or(left, right) => self.evaluate(left, item) || self.evaluate(right, item),
            FilterExpr::Not(expr) => !self.evaluate(expr, item),
            FilterExpr::Value(filter) => filter.filter(item),
            FilterExpr::Raw(_) => unreachable!(), // Should not happen after parsing
        }
    }
}

#[derive(Debug)]
struct NeedsBodyEvaluator;

impl<T> Evaluator<(), Box<dyn Filter<T>>> for NeedsBodyEvaluator {
    fn evaluate(&self, expr: &FilterExpr<Box<dyn Filter<T>>>, item: &()) -> bool {
        match expr {
            FilterExpr::And(left, right) => self.evaluate(left, item) || self.evaluate(right, item),
            FilterExpr::Or(left, right) => self.evaluate(left, item) || self.evaluate(right, item),
            FilterExpr::Not(expr) => self.evaluate(expr, item),
            FilterExpr::Value(filter) => filter.needs_body(),
            FilterExpr::Raw(_) => unreachable!(), // Should not happen after parsing
        }
    }
}

static FILTER_EVALUATOR: FilterEvaluator = FilterEvaluator;
static NEEDS_BODY_EVALUATOR: NeedsBodyEvaluator = NeedsBodyEvaluator;
