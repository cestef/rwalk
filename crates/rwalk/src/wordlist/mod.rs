use crate::{Result, engine::Task, utils::constants::DEFAULT_WORDLIST_KEY};
use cowstr::CowStr;
use crossbeam::deque::Injector;

use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use transformation::Transformer;
use url::Url;

pub mod filters;
pub mod processor;
pub mod transformation;

#[derive(Debug, Clone)]
pub struct Wordlist {
    pub words: Vec<CowStr>,
    pub key: CowStr,
}

impl Default for Wordlist {
    fn default() -> Self {
        Self {
            words: Vec::new(),
            key: DEFAULT_WORDLIST_KEY.into(),
        }
    }
}

impl Wordlist {
    pub fn new(key: CowStr) -> Self {
        Self {
            words: Vec::with_capacity(1024), // Pre-allocate reasonable default capacity
            key,
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.words.extend(other.words);
    }

    pub fn transform(&mut self, transformer: &Transformer<CowStr>) {
        self.words.par_iter_mut().for_each(|word| {
            transformer.apply(word);
        });
    }

    pub fn inject_into(&self, injector: &Injector<Task>, url: &Url, depth: usize) -> Result<()> {
        let base_url = url.to_string();
    
        self.words.par_iter().try_for_each(|word| {
            // Trim slashes to ensure exactly one slash between
            let base_trimmed = base_url.trim_end_matches('/');
            let word_trimmed = word.trim_start_matches('/');
    
            let full_url = format!("{}/{}", base_trimmed, word_trimmed);
    
            injector.push(Task::new_recursive(full_url, depth));
            Ok(())
        })
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn extend(&mut self, other: Self) {
        self.words.extend(other.words);
    }

    pub fn dedup(&mut self) {
        self.words.sort();
        self.words.dedup();
    }

    pub fn to_string(&self) -> String {
        self.words.join("\n")
    }
}
