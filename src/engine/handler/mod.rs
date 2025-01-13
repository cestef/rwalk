use crate::{filters::Filterer, worker::utils::RwalkResponse, Result};

use super::WorkerPool;

pub mod recursive;
pub mod template;

pub trait ResponseHandler: Send + Sync {
    fn handle(&self, response: RwalkResponse, pool: &WorkerPool) -> Result<()>;
    fn construct(filterer: Filterer<RwalkResponse>) -> Self
    where
        Self: Sized;
    fn init(&self, pool: &WorkerPool) -> Result<()>;
}
