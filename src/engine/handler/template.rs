use cowstr::CowStr;
use indicatif::ProgressBar;
use itertools::Itertools;
use owo_colors::OwoColorize;
use rayon::prelude::*;
use std::{collections::HashMap, time::Duration};

use crate::{
    engine::{Task, WorkerPool},
    error::{error, RwalkError},
    filters::Filterer,
    utils::format::{self, display_time, success},
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
        let pb = ProgressBar::new(0).with_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} {elapsed_precise}")
                .unwrap(),
        );
        pb.enable_steady_tick(Duration::from_millis(100));

        pb.set_message("Generating URLs for");

        let positions: HashMap<CowStr, Vec<_>> = wordlists
            .iter()
            .map(|w| {
                let positions = base_url
                    .match_indices(&*w.key)
                    .map(|(pos, _)| pos)
                    .collect::<Vec<_>>();
                (w.key.clone(), positions)
            })
            .collect();

        if positions.iter().all(|(_, v)| v.is_empty()) {
            return Err(error!("No template markers found in URL"));
        }

        let word_iters: Vec<_> = wordlists.iter().map(|wl| wl.words.iter()).collect();

        let combinations = word_iters
            .into_iter()
            .multi_cartesian_product()
            .par_bridge();

        let urls: Vec<String> = combinations
            .map(|words| {
                let mut url = base_url.to_string();
                for (wordlist, word) in wordlists.iter().zip(words) {
                    if let Some(positions) = positions.get(&wordlist.key) {
                        for &pos in positions {
                            url.replace_range(pos..pos + wordlist.key.len(), word);
                        }
                    }
                }
                url
            })
            .collect();

        pb.finish_and_clear();

        success!(
            "Generated {} URLs in {}",
            urls.len().to_string().bold(),
            display_time(pb.elapsed().as_nanos())
        );

        Ok(urls)
    }
}
