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
        if !is_directory(&response) {
            for wordlist in pool.wordlists.iter() {
                wordlist.inject_into(&pool.global_queue, &response.url)?;
            }
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
        for wordlist in pool.wordlists.iter() {
            wordlist.inject_into(&pool.global_queue, &pool.base_url)?;
        }
        Ok(())
    }
}
