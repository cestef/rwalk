use rayon::prelude::*;
use std::sync::Arc;

use crate::{
    engine::{Task, WorkerPool},
    error::{error, RwalkError},
    filters::Filterer,
    utils::format,
    wordlist::Wordlist,
    worker::utils::RwalkResponse,
    Result,
};

use super::ResponseHandler;

pub struct TemplateHandler {
    filterer: Filterer<RwalkResponse>,
}

impl ResponseHandler for TemplateHandler {
    fn handle(&self, response: RwalkResponse, pool: &WorkerPool) -> Result<()> {
        pool.pb.println(format::response(&response));
        Ok(())
    }

    fn construct(filterer: Filterer<RwalkResponse>) -> Self
    where
        Self: Sized,
    {
        Self { filterer }
    }

    fn init(&self, pool: &WorkerPool) -> Result<()> {
        let urls = self.generate_urls(&pool.wordlists, &pool.config.base_url.to_string())?;

        // Push URLs to queue in parallel chunks
        urls.par_chunks(pool.config.threads).for_each(|chunk| {
            for url in chunk {
                pool.global_queue.push(Task::new(url.to_string(), 0));
            }
        });

        Ok(())
    }
}

impl TemplateHandler {
    fn generate_urls(&self, wordlists: &Vec<Wordlist>, base_url: &str) -> Result<Vec<String>> {
        // Find all template markers and their positions in the URL
        let template_positions: Vec<_> = base_url.match_indices('$').map(|(pos, _)| pos).collect();

        if template_positions.is_empty() {
            return Ok(vec![base_url.to_string()]);
        }

        // Get the wordlist that corresponds to the '$' marker
        let wordlist = wordlists
            .iter()
            .find(|w| w.key == "$")
            .ok_or_else(|| error!("No wordlist found for template marker '$'"))?;

        let wordlist = Arc::new(wordlist);
        let base_url = Arc::new(base_url.to_string());
        let template_positions = Arc::new(template_positions);

        // Calculate total combinations
        let total_combinations = wordlist.words.len().pow(template_positions.len() as u32);

        // Split work into chunks
        let chunk_size = (total_combinations / rayon::current_num_threads()).max(1);

        // Generate URLs in parallel
        let urls: Vec<String> = (0..total_combinations)
            .into_par_iter()
            .chunks(chunk_size)
            .flat_map(|chunk| {
                let wordlist = Arc::clone(&wordlist);
                let base_url = Arc::clone(&base_url);
                let template_positions = Arc::clone(&template_positions);

                chunk
                    .into_iter()
                    .map(move |i| {
                        let mut combination = Vec::new();
                        let mut n = i;

                        for _ in 0..template_positions.len() {
                            let word_idx = n % wordlist.words.len();
                            combination.push(&wordlist.words[word_idx]);
                            n /= wordlist.words.len();
                        }

                        let mut url = (*base_url).clone();
                        let mut offset = 0;

                        for (pos, word) in template_positions.iter().zip(combination) {
                            let pos = *pos + offset;
                            url.replace_range(pos..pos + 1, word);
                            offset += word.len() - 1;
                        }

                        url
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        println!("Generated {} URLs", urls.len());
        Ok(urls)
    }
}
