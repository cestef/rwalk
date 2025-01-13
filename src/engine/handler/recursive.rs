use rayon::prelude::*;
use std::sync::Arc;

use crate::{
    engine::WorkerPool,
    filters::Filterer,
    worker::utils::{is_directory, RwalkResponse},
    Result,
};

use super::ResponseHandler;

pub struct RecursiveHandler {
    filterer: Filterer<RwalkResponse>,
}

impl ResponseHandler for RecursiveHandler {
    fn handle(&self, response: RwalkResponse, pool: &WorkerPool) -> Result<()> {
        // If it's a directory and passes filters, we should recursively scan it
        if is_directory(&response) {
            let pool = Arc::new(pool);
            let response = Arc::new(response);

            // Process wordlists in parallel
            pool.wordlists.par_iter().try_for_each(|wordlist| {
                let pool = Arc::clone(&pool);
                let response = Arc::clone(&response);
                wordlist.inject_into(&pool.global_queue, &response.url)
            })?;
        }

        Ok(())
    }

    fn construct(filterer: Filterer<RwalkResponse>) -> Self
    where
        Self: Sized,
    {
        Self { filterer }
    }

    fn init(&self, pool: &WorkerPool) -> Result<()> {
        let pool = Arc::new(pool);

        // Process initial wordlists in parallel
        pool.wordlists.par_iter().try_for_each(|wordlist| {
            let pool = Arc::clone(&pool);
            wordlist.inject_into(&pool.global_queue, &pool.config.base_url)
        })?;

        Ok(())
    }
}
