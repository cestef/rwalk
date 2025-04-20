use crate::{
    cli::Opts,
    error::{Result, RwalkError},
    filters::Filterer,
    wordlist::{
        filters::WordlistFilterRegistry,
        transformation::{Transformer, WordlistTransformerRegistry},
        Wordlist,
    },
};
use cowstr::CowStr;
use dashmap::{DashMap, DashSet};
use tracing::debug;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
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
        let filterer = Arc::new(self.create_filterer()?);

        // Process wordlists concurrently
        for (path, key) in &self.opts.wordlists {
            let path = path.clone();
            let key = key.clone();
            let transformer = Arc::new(self.create_transformer(&key)?);
            let shared_words = Arc::clone(&shared_words);
            let filterer = Arc::clone(&filterer);
            set.spawn(async move {
                let wordlist = Self::process_single_wordlist(
                    path.into(),
                    key.into(),
                    &transformer,
                    &filterer,
                    &shared_words,
                )
                .await?;
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
        path: CowStr,
        key: CowStr,
        transformer: &Transformer<String>,
        filterer: &Filterer<(CowStr, CowStr)>,
        shared_words: &DashMap<CowStr, DashSet<CowStr>>,
    ) -> Result<Wordlist> {
        debug!("Processing wordlist: {}", path);
        let path = PathBuf::from(&*path).canonicalize()?;
        debug!("Canonicalized path: {}", path.display());
        let file = File::open(&*path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut words = Vec::new();

        while let Some(line) = lines.next_line().await? {
            if !line.trim().is_empty() {
                let mut word = line;
                transformer.apply(&mut word);
                let word: CowStr = word.into();
                // Only add if not seen before for this key
                if shared_words
                    .entry(key.clone())
                    .or_insert_with(DashSet::new)
                    .insert(word.clone())
                    && filterer.filter(&(key.clone(), word.clone()))?
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

    fn create_filterer(&self) -> Result<Filterer<(CowStr, CowStr)>> {
        let filter = if let Some(ref filter) = self.opts.wordlist_filter {
            Some(WordlistFilterRegistry::construct(filter)?)
        } else {
            None
        };

        Ok(Filterer::new(filter))
    }
}
