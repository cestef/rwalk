use super::expression::{Evaluator, FilterExpr, Parser};
use super::Filter;
use crate::types::IntRange;
use crate::worker::utils::SendableResponse;
use crate::Result;

#[derive(Debug, Clone)]
pub struct LengthFilter {
    expr: FilterExpr<IntRange<u16>>,
}

#[derive(Debug)]
struct LengthValueParser;

impl Filter<SendableResponse> for LengthFilter {
    fn filter(&self, item: &SendableResponse) -> bool {
        LengthEvaluator.evaluate(&self.expr, item)
    }

    fn name() -> &'static str {
        "length"
    }

    fn aliases() -> &'static [&'static str] {
        &["s", "code"]
    }

    fn construct(arg: &str) -> Result<Box<dyn Filter<SendableResponse>>>
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

impl Evaluator<SendableResponse, IntRange<u16>> for LengthEvaluator {
    fn evaluate(&self, expr: &FilterExpr<IntRange<u16>>, item: &SendableResponse) -> bool {
        match expr {
            FilterExpr::And(left, right) => self.evaluate(left, item) && self.evaluate(right, item),
            FilterExpr::Or(left, right) => self.evaluate(left, item) || self.evaluate(right, item),
            FilterExpr::Not(expr) => !self.evaluate(expr, item),
            FilterExpr::Value(range) => {
                if let Some(body) = &item.body {
                    range.contains(body.len() as u16)
                } else {
                    false
                }
            }
            FilterExpr::Raw(_) => unreachable!(), // Should not happen after parsing
        }
    }
}
