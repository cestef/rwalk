use super::Filter;
use crate::{worker::utils::RwalkResponse, Result};

#[derive(Debug, Clone)]
pub struct StartsFilter {
    substr: String,
}

#[derive(Debug)]
struct LengthValueParser;

impl Filter<RwalkResponse> for StartsFilter {
    fn filter(&self, item: &RwalkResponse) -> bool {
        item.body
            .as_ref()
            .map_or(false, |e| e.starts_with(&self.substr))
    }

    fn needs_body(&self) -> bool {
        true
    }

    fn name() -> &'static str {
        "starts"
    }

    fn aliases() -> &'static [&'static str] {
        &["s"]
    }

    fn construct(arg: &str) -> Result<Box<dyn Filter<RwalkResponse>>>
    where
        Self: Sized,
    {
        Ok(Box::new(StartsFilter {
            substr: arg.to_string(),
        }))
    }
}
