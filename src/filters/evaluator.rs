use crate::filters::expression::FilterExpr;
use crate::Result;
pub struct GenericEvaluator<V, T> {
    filter_fn: fn(&T, &V) -> Result<bool>,
    _phantom: std::marker::PhantomData<V>,
}

impl<V, T> GenericEvaluator<V, T> {
    pub fn new(filter_fn: fn(&T, &V) -> Result<bool>) -> Self {
        Self {
            filter_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<V, T> Evaluator<T, V> for GenericEvaluator<V, T> {
    fn evaluate(&self, expr: &FilterExpr<V>, item: &T) -> Result<bool> {
        use FilterExpr::*;
        match expr {
            And(left, right) => Ok(self.evaluate(left, item)? && self.evaluate(right, item)?),
            Or(left, right) => Ok(self.evaluate(left, item)? || self.evaluate(right, item)?),
            Not(expr) => Ok(!self.evaluate(expr, item)?),
            Value(sub) => (self.filter_fn)(item, sub),
            Raw(e) => unreachable!("{e}"), // Should not happen after parsing
        }
    }
}

pub trait Evaluator<T, V> {
    fn evaluate(&self, expr: &FilterExpr<V>, item: &T) -> Result<bool>;
}
