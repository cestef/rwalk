use super::Filter;
use crate::Result;

#[derive(Debug, Clone)]
pub struct StartsFilter {
    substr: String,
}

#[derive(Debug)]
struct LengthValueParser;

impl Filter<String> for StartsFilter {
    fn filter(&self, item: &String) -> bool {
        item.starts_with(&self.substr)
    }

    fn name() -> &'static str {
        "starts"
    }

    fn aliases() -> &'static [&'static str] {
        &["s"]
    }

    fn construct(arg: &str, _: Option<usize>) -> Result<Box<dyn Filter<String>>>
    where
        Self: Sized,
    {
        Ok(Box::new(StartsFilter {
            substr: arg.to_string(),
        }))
    }
}
