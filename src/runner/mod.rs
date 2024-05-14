pub mod classic;
pub mod client;
pub mod extract;
pub mod filters;
pub mod recursive;
pub mod scripting;
pub mod spider;
pub mod wordlists;

use std::future::Future;

use anyhow::Result;

pub trait Runner {
    fn run(self) -> impl Future<Output = Result<()>> + Send;
}
