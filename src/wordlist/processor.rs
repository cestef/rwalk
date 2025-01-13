use crate::{
    cli::Opts,
    error::{Result, RwalkError},
    wordlist::{
        transformation::{Transformer, WordlistTransformerRegistry},
        Wordlist,
    },
};
use dashmap::{DashMap, DashSet};

use std::sync::Arc;
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
        let mut merged = Vec::with_capacity(self.opts.wordlists.len());
        let shared_words = Arc::new(DashMap::new());

        // Process wordlists concurrently
        for (path, key) in &self.opts.wordlists {
            let path = path.clone();
            let key = key.clone();
            let transformer = Arc::new(self.create_transformer(&key)?);
            let shared_words = Arc::clone(&shared_words);

            set.spawn(async move {
                let wordlist =
                    Self::process_single_wordlist(path, key, &transformer, &shared_words).await?;
                Ok::<Wordlist, RwalkError>(wordlist)
            });
        }

        // Collect results
        while let Some(result) = set.join_next().await {
            let wordlist = result??;
            if !wordlist.is_empty() {
                merged.push(wordlist);
            }
        }

        Ok(merged)
    }

    async fn process_single_wordlist(
        path: String,
        key: String,
        transformer: &Transformer<String>,
        shared_words: &DashMap<String, DashSet<String>>,
    ) -> Result<Wordlist> {
        let file = File::open(&path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut words = Vec::new();

        while let Some(line) = lines.next_line().await? {
            if !line.trim().is_empty() {
                let mut word = line;
                transformer.apply(&mut word);

                // Only add if not seen before for this key
                if shared_words
                    .entry(key.clone())
                    .or_insert_with(DashSet::new)
                    .insert(word.clone())
                {
                    words.push(word);
                }
            }
        }

        Ok(Wordlist { words, key })
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
}
