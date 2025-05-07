use indicatif::{ProgressBar, ProgressIterator};
use itertools::Itertools;
use owo_colors::OwoColorize;
use rayon::prelude::*;
use std::time::Duration;

use crate::{
    Result,
    engine::{Task, WorkerPool},
    error::{RwalkError, error},
    filters::Filterer,
    success,
    utils::format::{self, display_time},
    wordlist::Wordlist,
    worker::utils::RwalkResponse,
};

use super::ResponseHandler;

pub struct TemplateHandler {
    filterer: Filterer<RwalkResponse>,
}

impl ResponseHandler for TemplateHandler {
    fn handle(&self, response: RwalkResponse, pool: &WorkerPool) -> Result<()> {
        pool.pb
            .println(format::response(&response, &pool.config.show));
        Ok(())
    }

    fn construct(filterer: Filterer<RwalkResponse>) -> Self
    where
        Self: Sized,
    {
        Self { filterer }
    }

    fn init(&self, pool: &WorkerPool) -> Result<()> {
        let urls = self.generate_urls(&pool.wordlists, pool.config.base_url.as_ref())?;

        urls.par_chunks(pool.config.threads).for_each(|chunk| {
            for url in chunk {
                pool.global_queue.push(Task::new(url.to_string(), 0));
            }
        });

        Ok(())
    }
}

impl TemplateHandler {
    fn generate_urls(&self, wordlists: &[Wordlist], base_url: &str) -> Result<Vec<String>> {
        let pb = ProgressBar::new(0).with_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} {elapsed_precise} ({human_pos:>7.dim}/{human_len:7.dim} | {per_sec:.dimmed})")
                .unwrap(),
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Generating URLs for");

        // Pre-compute all segments of the base URL between replacement points
        let mut segments = Vec::new();
        let mut last_end = 0;
        let mut all_positions = Vec::new();

        for (wl_idx, wordlist) in wordlists.iter().enumerate() {
            let positions: Vec<_> = base_url
                .match_indices(&*wordlist.key)
                .map(|(pos, _)| (pos, wl_idx, wordlist.key.len()))
                .collect();

            if !positions.is_empty() {
                all_positions.extend(positions);
            }
        }

        if all_positions.is_empty() {
            return Err(error!("No template markers found in URL"));
        }

        // Sort positions to process them in order
        all_positions.sort_by_key(|&(pos, _, _)| pos);

        // Create URL segments
        for &(pos, _, len) in &all_positions {
            if pos > last_end {
                segments.push(base_url[last_end..pos].to_string());
            }
            segments.push(String::new()); // Placeholder for word insertion
            last_end = pos + len;
        }

        // Add the last segment if needed
        if last_end < base_url.len() {
            segments.push(base_url[last_end..].to_string());
        }

        let word_iters = wordlists.iter().map(|wl| wl.words.iter());
        let total_length: usize = word_iters.clone().map(|iter| iter.len()).product();
        pb.set_length(total_length as u64);

        // Create a mapping from wordlist index to positions in segments vector
        let mut wl_to_segment_positions: Vec<Vec<usize>> = vec![Vec::new(); wordlists.len()];
        for (seg_idx, &(_, wl_idx, _)) in all_positions.iter().enumerate() {
            wl_to_segment_positions[wl_idx].push(seg_idx * 2 + 1); // +1 because segments alternate between fixed text and placeholders
        }

        let combinations = word_iters.multi_cartesian_product();
        let urls: Vec<String> = combinations
            .progress_with(pb.clone())
            .map(|words| {
                let mut url_segments = segments.clone();

                // Fill in the placeholders
                for (wl_idx, word) in words.iter().enumerate() {
                    for &pos in &wl_to_segment_positions[wl_idx] {
                        url_segments[pos] = word.to_string();
                    }
                }

                let url = url_segments.concat();
                url
            })
            .collect();

        pb.finish_and_clear();

        success!(
            "Generated {} URLs in {}",
            urls.len().to_string().bold(),
            display_time(pb.elapsed().as_micros() as i64)
        );

        Ok(urls)
    }
}
