use super::Filter;
use crate::filters::expression::{Evaluator, FilterExpr, Parser};
use crate::types::IntRange;
use crate::worker::utils::RwalkResponse;
use crate::Result;

type Range = IntRange<usize>;

#[derive(Debug, Clone)]
pub struct LengthFilter {
    expr: FilterExpr<Range>,
}

#[derive(Debug)]
struct LengthValueParser;

impl Filter<RwalkResponse> for LengthFilter {
    fn filter(&self, item: &RwalkResponse) -> bool {
        LengthEvaluator.evaluate(&self.expr, item)
    }

    fn needs_body(&self) -> bool {
        true
    }

    fn name() -> &'static str {
        "length"
    }

    fn aliases() -> &'static [&'static str] {
        &["l"]
    }

    fn construct(arg: &str) -> Result<Box<dyn Filter<RwalkResponse>>>
    where
        Self: Sized,
    {
        let mut parser = Parser::new(arg);
        let raw_expr = parser.parse::<String>()?;

        // Transform raw expressions into IntRange expressions
        let expr = raw_expr.try_map(|raw: String| raw.parse())?;

        Ok(Box::new(LengthFilter { expr }))
    }
}

#[derive(Debug)]
struct LengthEvaluator;

impl Evaluator<RwalkResponse, Range> for LengthEvaluator {
    fn evaluate(&self, expr: &FilterExpr<Range>, item: &RwalkResponse) -> bool {
        match expr {
            FilterExpr::And(left, right) => self.evaluate(left, item) && self.evaluate(right, item),
            FilterExpr::Or(left, right) => self.evaluate(left, item) || self.evaluate(right, item),
            FilterExpr::Not(expr) => !self.evaluate(expr, item),
            FilterExpr::Value(range) => {
                if let Some(body) = &item.body {
                    range.contains(body.len())
                } else {
                    false
                }
            }
            FilterExpr::Raw(_) => unreachable!(), // Should not happen after parsing
        }
    }
}
