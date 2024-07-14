pub mod classic;
pub mod client;
pub mod filters;
pub mod recursive;
pub mod spider;
pub mod wordlists;

use std::future::Future;

use color_eyre::eyre::Result;

pub trait Runner {
    fn run(self) -> impl Future<Output = Result<()>> + Send;
}
