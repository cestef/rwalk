use rayon::prelude::*;
use std::sync::Arc;

use crate::{
    Result,
    engine::WorkerPool,
    filters::Filterer,
    utils::{directory, format},
    worker::utils::RwalkResponse,
};

use super::ResponseHandler;

pub struct RecursiveHandler {
    filterer: Filterer<RwalkResponse>,
}

impl ResponseHandler for RecursiveHandler {
    fn handle(&self, response: RwalkResponse, pool: &WorkerPool) -> Result<()> {
        // If it's a directory and passes filters, we should recursively scan it
        if (response.depth as usize) < pool.config.max_depth {
            if pool.config.force_recursion || directory::check(&response) {
                pool.pb
                    .println(format::response(&response, &pool.config.show));

                let total_entries: u64 = pool.wordlists.iter().map(|w| w.len() as u64).sum();

                pool.wordlists.par_iter().try_for_each(|wordlist| {
                    wordlist.inject_into(
                        &pool.global_queue,
                        &response.url,
                        response.depth as usize + 1,
                    )
                })?;

                pool.pb
                    .set_length(pool.pb.length().unwrap() + total_entries);
            } else {
                pool.pb.println(format::skip(
                    &response,
                    format::SkipReason::NonDirectory,
                    &pool.config.show,
                ));
            }
        } else {
            pool.pb.println(format::skip(
                &response,
                format::SkipReason::MaxDepth,
                &pool.config.show,
            ));
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
            wordlist.inject_into(&pool.global_queue, &pool.config.base_url, 0)
        })?;

        Ok(())
    }
}
