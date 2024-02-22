pub mod classic;
pub mod client;
pub mod filters;
pub mod recursive;
pub mod wordlists;

use std::future::Future;

use anyhow::Result;

pub trait Runner {
    fn run(self) -> impl Future<Output = Result<()>> + Send;
}
