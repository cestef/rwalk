use super::Filter;
use crate::filters::expression::{Evaluator, ExprParser, FilterExpr};
use crate::types::IntRange;
use crate::Result;

type Range = IntRange<usize>;

#[derive(Debug, Clone)]
pub struct LengthFilter {
    expr: FilterExpr<Range>,
}

#[derive(Debug)]
struct LengthValueParser;

impl Filter<String> for LengthFilter {
    fn filter(&self, item: &String) -> bool {
        LengthEvaluator.evaluate(&self.expr, item)
    }

    fn name() -> &'static str {
        "length"
    }

    fn aliases() -> &'static [&'static str] {
        &["l"]
    }

    fn construct(arg: &str, _: Option<usize>) -> Result<Box<dyn Filter<String>>>
    where
        Self: Sized,
    {
        let mut parser = ExprParser::new(arg);
        let raw_expr = parser.parse::<String>()?;

        // Transform raw expressions into IntRange expressions
        let expr = raw_expr.try_map(|raw: String| raw.parse())?;

        Ok(Box::new(LengthFilter { expr }))
    }
}

#[derive(Debug)]
struct LengthEvaluator;

impl Evaluator<String, Range> for LengthEvaluator {
    fn evaluate(&self, expr: &FilterExpr<Range>, item: &String) -> bool {
        match expr {
            FilterExpr::And(left, right) => self.evaluate(left, item) && self.evaluate(right, item),
            FilterExpr::Or(left, right) => self.evaluate(left, item) || self.evaluate(right, item),
            FilterExpr::Not(expr) => !self.evaluate(expr, item),
            FilterExpr::Value(range) => range.contains(item.len()),
            FilterExpr::Raw(_) => unreachable!(), // Should not happen after parsing
        }
    }
}
