use cowstr::CowStr;
use indicatif::{ProgressBar, ProgressIterator};
use itertools::Itertools;
use owo_colors::OwoColorize;
use rayon::prelude::*;
use std::time::Duration;
use tracing::debug;

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
type HeaderMap = std::collections::HashMap<String, String>;

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
        let results = self.generate_templates(
            &pool.wordlists,
            pool.config.base_url.as_ref(),
            pool.config.data.as_deref(),
            pool.config.headers.as_ref().map(|h| {
                h.iter()
                    .flat_map(|(_, v)| v.iter().map(|(k, v)| (k.clone(), v.clone())))
                    .collect::<HeaderMap>()
            }),
        )?;

        results.par_chunks(pool.config.threads).for_each(|chunk| {
            for (url, data, headers) in chunk {
                pool.global_queue.push(Task::new_template(
                    url.to_string(),
                    data.clone(),
                    headers.clone(),
                ));
            }
        });

        Ok(())
    }
}

impl TemplateHandler {
    // Generic function to process a template string and find positions
    fn process_template_string(
        &self,
        template: &str,
        wordlists: &[Wordlist],
    ) -> (Vec<String>, Vec<Vec<usize>>) {
        // Find all positions of template keys
        let mut template_positions = Vec::new();

        for (wl_idx, wordlist) in wordlists.iter().enumerate() {
            let positions: Vec<_> = template
                .match_indices(&*wordlist.key)
                .map(|(pos, _)| (pos, wl_idx, wordlist.key.len()))
                .collect();

            if !positions.is_empty() {
                template_positions.extend(positions);
            }
        }

        // Sort positions by their location in the template
        template_positions.sort_by_key(|&(pos, _, _)| pos);

        // Create segments
        let mut segments = Vec::new();
        let mut last_end = 0;

        for &(pos, _, len) in &template_positions {
            if pos > last_end {
                segments.push(template[last_end..pos].to_string());
            }
            segments.push(String::new()); // Placeholder for word insertion
            last_end = pos + len;
        }

        // Add the last segment if needed
        if last_end < template.len() {
            segments.push(template[last_end..].to_string());
        }

        // Create mapping from wordlist index to segment positions
        let mut wl_to_positions: Vec<Vec<usize>> = vec![Vec::new(); wordlists.len()];

        for (seg_idx, &(_, wl_idx, _)) in template_positions.iter().enumerate() {
            wl_to_positions[wl_idx].push(seg_idx * 2 + 1); // +1 because segments alternate
        }

        (segments, wl_to_positions)
    }

    fn fill_template(
        &self,
        segments: &[String],
        wl_to_positions: &[Vec<usize>],
        words: &[&CowStr],
    ) -> String {
        let mut result = segments.to_vec();
        // debug!(
        //     "fill_template: segments: {:?}, wl_to_positions: {:?}",
        //     segments, wl_to_positions
        // );
        // Fill in the placeholders
        for (wl_idx, word) in words.iter().enumerate() {
            if wl_idx < wl_to_positions.len() {
                for &pos in &wl_to_positions[wl_idx] {
                    if pos < result.len() {
                        result[pos] = word.to_string();
                    } else {
                        result.push(word.to_string());
                    }
                }
            }
        }

        result.concat()
    }

    fn generate_templates(
        &self,
        wordlists: &[Wordlist],
        base_url: &str,
        base_data: Option<&str>,
        base_headers: Option<HeaderMap>,
    ) -> Result<Vec<(String, Option<String>, HeaderMap)>> {
        let pb = ProgressBar::new(0).with_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} {elapsed_precise} ({human_pos:>7.dim}/{human_len:7.dim} | {per_sec:.dimmed})")
                .unwrap(),
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Generating templates for");

        // Process the URL template
        let (url_segments, url_wl_positions) = self.process_template_string(base_url, wordlists);

        // Process the data template if it exists
        let (data_segments, data_wl_positions) = if let Some(data) = base_data {
            self.process_template_string(data, wordlists)
        } else {
            (Vec::new(), Vec::new())
        };

        // Process header templates if they exist
        let mut header_templates = Vec::new();
        let mut has_header_templates = false;

        if let Some(ref headers) = base_headers {
            // debug!("Processing headers: {:#?}", headers);
            for (header_name, header_value) in headers {
                let (segments, positions) = self.process_template_string(&header_value, wordlists);
                let has_template = !positions.iter().all(|v| v.is_empty());
                has_header_templates |= has_template;
                header_templates.push((header_name.clone(), segments, positions));
            }
        } else {
            debug!("No headers provided");
        }

        // Check if we found any template markers
        let has_url_templates = !url_wl_positions.iter().all(|v| v.is_empty());
        let has_data_templates = !data_wl_positions.iter().all(|v| v.is_empty());

        if !has_url_templates && !has_data_templates && !has_header_templates {
            return Err(error!("No template markers found in URL, data, or headers"));
        }

        // Calculate total combinations
        let word_iters = wordlists.iter().map(|wl| wl.words.iter());
        let total_length: usize = word_iters.clone().map(|iter| iter.len()).product();
        pb.set_length(total_length as u64);

        // Generate all combinations
        let combinations = word_iters.multi_cartesian_product();

        let results: Vec<(String, Option<String>, HeaderMap)> = combinations
            .progress_with(pb.clone())
            .map(|words| {
                // Generate URL from template
                let url = self.fill_template(&url_segments, &url_wl_positions, &words);

                // Generate data from template if it exists
                let data = if !data_segments.is_empty() {
                    Some(self.fill_template(&data_segments, &data_wl_positions, &words))
                } else if base_data.is_some() {
                    base_data.map(String::from)
                } else {
                    None
                };

                // Generate headers from templates
                let mut headers = HeaderMap::new();
                if let Some(ref base_headers) = base_headers {
                    // First copy any non-templated headers
                    for (key, value) in base_headers {
                        if !header_templates.iter().any(|(name, _, _)| name == key) {
                            headers.insert(key.clone(), value.clone());
                        }
                    }

                    // Then process templated headers
                    for (header_name, segments, positions) in &header_templates {
                        let value = self.fill_template(segments, positions, &words);
                        headers.insert(header_name.clone(), value);
                    }
                }

                (url, data, headers)
            })
            .collect();

        pb.finish_and_clear();

        success!(
            "Generated {} combinations in {}",
            results.len().to_string().bold(),
            display_time(pb.elapsed().as_micros() as i64)
        );

        Ok(results)
    }
}
