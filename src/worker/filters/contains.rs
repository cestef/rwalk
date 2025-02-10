use super::Filter;
use crate::{worker::utils::RwalkResponse, Result};

#[derive(Debug, Clone)]
pub struct ContainsFilter {
    substr: String,
}

#[derive(Debug)]
struct LengthValueParser;

impl Filter<RwalkResponse> for ContainsFilter {
    fn filter(&self, item: &RwalkResponse) -> bool {
        item.body
            .as_ref()
            .map_or(false, |e| e.contains(&self.substr))
    }

    fn needs_body(&self) -> bool {
        true
    }

    fn name() -> &'static str {
        "contains"
    }

    fn aliases() -> &'static [&'static str] {
        &["c", "has"]
    }

    fn construct(arg: &str) -> Result<Box<dyn Filter<RwalkResponse>>>
    where
        Self: Sized,
    {
        Ok(Box::new(ContainsFilter {
            substr: arg.to_string(),
        }))
    }
}
