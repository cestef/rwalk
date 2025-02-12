use crate::filters::expression::FilterExpr;

pub struct GenericEvaluator<V, T> {
    filter_fn: fn(&T, &V) -> bool,
    _phantom: std::marker::PhantomData<V>,
}

impl<V, T> GenericEvaluator<V, T> {
    pub fn new(filter_fn: fn(&T, &V) -> bool) -> Self {
        Self {
            filter_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<V, T> Evaluator<T, V> for GenericEvaluator<V, T> {
    fn evaluate(&self, expr: &FilterExpr<V>, item: &T) -> bool {
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

pub trait Evaluator<T, V> {
    fn evaluate(&self, expr: &FilterExpr<V>, item: &T) -> bool;
}
