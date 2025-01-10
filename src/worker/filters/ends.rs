use super::Filter;
use crate::{worker::utils::RwalkResponse, Result};

#[derive(Debug, Clone)]
pub struct EndsFilter {
    substr: String,
}

#[derive(Debug)]
struct LengthValueParser;

impl Filter<RwalkResponse> for EndsFilter {
    fn filter(&self, item: &RwalkResponse) -> bool {
        item.body
            .as_ref()
            .map_or(false, |e| e.ends_with(&self.substr))
    }

    fn name() -> &'static str {
        "ends"
    }

    fn aliases() -> &'static [&'static str] {
        &["e"]
    }

    fn construct(arg: &str) -> Result<Box<dyn Filter<RwalkResponse>>>
    where
        Self: Sized,
    {
        Ok(Box::new(EndsFilter {
            substr: arg.to_string(),
        }))
    }
}
