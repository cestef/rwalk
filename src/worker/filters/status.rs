use eyre::Result;

use crate::{types::IntRange, worker::utils::SendableResponse};

use super::Filter;

#[derive(Debug, Clone)]
pub struct StatusFilter {
    statuses: Vec<IntRange<u16>>,
}

impl Filter<SendableResponse> for StatusFilter {
    fn filter(&self, item: &SendableResponse) -> bool {
        self.statuses
            .iter()
            .any(|range| range.contains(item.status))
    }

    fn name(&self) -> &'static str {
        "status"
    }

    fn aliases(&self) -> &[&'static str] {
        &["s"]
    }
    fn construct(arg: &str) -> Result<Box<dyn Filter<SendableResponse>>>
    where
        Self: Sized,
    {
        let statuses = arg
            .split(',')
            .map(|s| s.parse())
            .collect::<Result<Vec<IntRange<u16>>>>()?;
        Ok(Box::new(StatusFilter { statuses }))
    }
}
