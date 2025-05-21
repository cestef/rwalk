use crate::{
    cli::Opts,
    error,
    error::{Result, RwalkError},
    filters::Filterer,
    wordlist::{
        Wordlist,
        filters::WordlistFilterRegistry,
        transformation::{Transformer, WordlistTransformerRegistry},
    },
};
use cowstr::CowStr;
use dashmap::{DashMap, DashSet};
use owo_colors::OwoColorize;
use tracing::debug;

use std::{path::PathBuf, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    task::JoinSet,
};

pub struct WordlistProcessor<'a> {
    opts: &'a Opts,
}

impl<'a> WordlistProcessor<'a> {
    pub fn new(opts: &'a Opts) -> Self {
        Self { opts }
    }

    pub async fn process_wordlists(&self) -> Result<Vec<Wordlist>> {
        let mut set = JoinSet::new();
        let shared_words = Arc::new(DashMap::new());
        let filterer = Arc::new(self.create_filterer()?);

        // Process wordlists concurrently - just populate shared_words
        for (path, key) in &self.opts.wordlists {
            let path = path.clone();
            let key = key.clone();
            let transformer = Arc::new(self.create_transformer(&key)?);
            let shared_words = Arc::clone(&shared_words);
            let filterer = Arc::clone(&filterer);
            let include_comments = self.opts.include_comments;
            set.spawn(async move {
                Self::process_single_wordlist(
                    path.into(),
                    key.into(),
                    &transformer,
                    &filterer,
                    &shared_words,
                    include_comments,
                )
                .await
            });
        }

        // Wait for all processing to complete
        while let Some(result) = set.join_next().await {
            result??;
        }

        for (srcs, dest) in &self.opts.merge {
            let mut to_remove = Vec::new();

            for src in srcs.iter() {
                let src = shared_words
                    .get::<CowStr>(&src.key().into())
                    .ok_or_else(|| {
                        error!(
                            "Wordlist {} not found in shared words",
                            src.to_string().bold()
                        )
                    })?;
                debug!("Merging {} into {}", src.key(), dest);

                shared_words
                    .entry(dest.into())
                    .or_default()
                    .extend(src.value().iter().map(|word| word.clone()));

                to_remove.push(src.key().clone());
            }

            for key in to_remove {
                shared_words.remove(&key);
            }
        }

        let merged = shared_words
            .iter()
            .filter(|entry| !entry.value().is_empty())
            .map(|entry| {
                let key = entry.key().clone();
                let words = entry
                    .value()
                    .iter()
                    .map(|word| word.clone())
                    .collect::<Vec<CowStr>>();
                Wordlist { words, key }
            })
            .collect::<Vec<Wordlist>>();

        Ok(merged)
    }

    async fn process_single_wordlist(
        path: CowStr,
        key: CowStr,
        transformer: &Transformer<String>,
        filterer: &Filterer<(CowStr, CowStr)>,
        shared_words: &DashMap<CowStr, DashSet<CowStr>>,
        include_comments: bool,
    ) -> Result<()> {
        debug!("Processing wordlist: {}", path);
        let path = PathBuf::from(&*path)
            .canonicalize()
            .map_err(|e| crate::error!("Failed to open wordlist file {}: {}", path.bold(), e))?;
        debug!("Canonicalized path: {}", path.display());
        let file = File::open(&*path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if !line.trim().is_empty() {
                let processed_line = if include_comments {
                    Some(line)
                } else {
                    Self::strip_comments(&line)
                };

                if let Some(mut word) = processed_line {
                    transformer.apply(&mut word);
                    let word: CowStr = word.into();
                    // Add word to shared structure if it passes the filter
                    if filterer.filter(&(key.clone(), word.clone()))? {
                        shared_words.entry(key.clone()).or_default().insert(word);
                    }
                }
            }
        }

        Ok(())
    }

    fn create_transformer(&self, wordlist_key: &str) -> Result<Transformer<String>> {
        let transformers = self
            .opts
            .transforms
            .iter()
            .filter(|(keys, _, _)| {
                // Apply if no keys specified or if this wordlist's key is in the set
                keys.is_empty() || keys.contains(wordlist_key)
            })
            .map(|(_, name, arg)| WordlistTransformerRegistry::construct(name, arg.as_deref()))
            .collect::<Result<Vec<_>>>()?;

        Ok(Transformer::new(transformers))
    }

    fn create_filterer(&self) -> Result<Filterer<(CowStr, CowStr)>> {
        let filter = if let Some(ref filter) = self.opts.wordlist_filter {
            Some(WordlistFilterRegistry::construct(filter)?)
        } else {
            None
        };

        Ok(Filterer::new(filter))
    }

    // Ref: https://github.com/ffuf/ffuf/blob/57da720af7d1b66066cbbde685b49948f886b29c/pkg/input/wordlist.go#L173
    fn strip_comments(text: &str) -> Option<String> {
        // If the line starts with # (ignoring leading whitespace), return None
        if text.trim_start().starts_with('#') {
            return None;
        }

        // Find the position of "#" after a space
        if let Some(index) = text.find(" #") {
            Some(text[..index].to_string())
        } else {
            Some(text.to_string())
        }
    }
}
