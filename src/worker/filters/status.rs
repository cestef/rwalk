use crate::{
    filters::{
        expression::{Evaluator, FilterExpr, Parser},
        Filter,
    },
    types::IntRange,
    worker::utils::RwalkResponse,
    Result,
};

#[derive(Debug, Clone)]
pub struct StatusFilter {
    expr: FilterExpr<IntRange<u16>>,
}

impl Filter<RwalkResponse> for StatusFilter {
    fn filter(&self, item: &RwalkResponse) -> bool {
        StatusEvaluator.evaluate(&self.expr, item)
    }

    fn name() -> &'static str {
        "status"
    }

    fn aliases() -> &'static [&'static str] {
        &["s", "code"]
    }

    fn construct(arg: &str) -> Result<Box<dyn Filter<RwalkResponse>>>
    where
        Self: Sized,
    {
        let mut parser = Parser::new(arg);
        let raw_expr = parser.parse::<String>()?;

        // Transform raw expressions into IntRange expressions
        let expr = raw_expr.try_map(|raw: String| raw.parse())?;

        Ok(Box::new(StatusFilter { expr }))
    }
}

#[derive(Debug)]
struct StatusEvaluator;

impl Evaluator<RwalkResponse, IntRange<u16>> for StatusEvaluator {
    fn evaluate(&self, expr: &FilterExpr<IntRange<u16>>, item: &RwalkResponse) -> bool {
        match expr {
            FilterExpr::And(left, right) => self.evaluate(left, item) && self.evaluate(right, item),
            FilterExpr::Or(left, right) => self.evaluate(left, item) || self.evaluate(right, item),
            FilterExpr::Not(expr) => !self.evaluate(expr, item),
            FilterExpr::Value(range) => range.contains(item.status),
            FilterExpr::Raw(_) => unreachable!(), // Should not happen after parsing
        }
    }
}
