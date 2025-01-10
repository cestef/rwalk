use super::Filter;
use crate::Result;

#[derive(Debug, Clone)]
pub struct EndsFilter {
    substr: String,
}

#[derive(Debug)]
struct LengthValueParser;

impl Filter<String> for EndsFilter {
    fn filter(&self, item: &String) -> bool {
        item.ends_with(&self.substr)
    }

    fn name() -> &'static str {
        "ends"
    }

    fn aliases() -> &'static [&'static str] {
        &["e"]
    }

    fn construct(arg: &str) -> Result<Box<dyn Filter<String>>>
    where
        Self: Sized,
    {
        Ok(Box::new(EndsFilter {
            substr: arg.to_string(),
        }))
    }
}
