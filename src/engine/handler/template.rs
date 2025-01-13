use itertools::Itertools;

use crate::{
    engine::WorkerPool, filters::Filterer, wordlist::Wordlist, worker::utils::RwalkResponse, Result,
};

use super::ResponseHandler;

pub struct TemplateHandler {
    filterer: Filterer<RwalkResponse>,
}

impl ResponseHandler for TemplateHandler {
    fn handle(&self, response: RwalkResponse, pool: &WorkerPool) -> Result<()> {
        Ok(())
    }
    fn construct(filterer: Filterer<RwalkResponse>) -> Self
    where
        Self: Sized,
    {
        Self { filterer }
    }

    fn init(&self, pool: &WorkerPool) -> Result<()> {
        let urls = self.generate_urls(&pool.wordlists, pool.base_url.to_string());

        for url in urls {
            pool.global_queue.push(url);
        }
        Ok(())
    }
}

impl TemplateHandler {
    /// Generate all possible URLs using a cartesian product of the wordlists
    fn generate_urls(&self, wordlists: &Vec<Wordlist>, url: String) -> Vec<String> {
        let products = wordlists
            .iter()
            .map(|w| {
                w.words
                    .iter()
                    .map(|word| (w.key.clone(), word.clone()))
                    .collect::<Vec<_>>()
            })
            .multi_cartesian_product()
            .collect::<Vec<_>>();
        let mut urls = vec![];
        for product in &products {
            let mut url = url.clone();
            for (k, v) in product {
                url = url.replace(k, v);
            }
            urls.push(url);
        }
        urls
    }
}
