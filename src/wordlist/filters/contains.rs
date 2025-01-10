use super::Filter;
use crate::Result;

#[derive(Debug, Clone)]
pub struct ContainsFilter {
    substr: String,
}

#[derive(Debug)]
struct LengthValueParser;

impl Filter<String> for ContainsFilter {
    fn filter(&self, item: &String) -> bool {
        item.contains(&self.substr)
    }

    fn name() -> &'static str {
        "contains"
    }

    fn aliases() -> &'static [&'static str] {
        &["c", "has"]
    }

    fn construct(arg: &str) -> Result<Box<dyn Filter<String>>>
    where
        Self: Sized,
    {
        Ok(Box::new(ContainsFilter {
            substr: arg.to_string(),
        }))
    }
}
