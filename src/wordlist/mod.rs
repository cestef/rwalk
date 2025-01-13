use crate::Result;
use crossbeam::deque::Injector;

use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use transformation::Transformer;
use url::Url;

pub mod filters;
pub mod processor;
pub mod transformation;

#[derive(Debug, Clone)]
pub struct Wordlist {
    pub words: Vec<String>,
    pub key: String,
}
impl Wordlist {
    pub fn new(key: String) -> Self {
        Self {
            words: Vec::with_capacity(1024), // Pre-allocate reasonable default capacity
            key,
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.words.extend(other.words);
    }

    pub fn transform(&mut self, transformer: &Transformer<String>) {
        self.words.par_iter_mut().for_each(|word| {
            transformer.apply(word);
        });
    }

    pub fn inject_into(&self, injector: &Injector<String>, url: &Url) -> Result<()> {
        let base_url = url.clone();
        self.words.par_iter().try_for_each(|word| {
            let mut url = base_url.clone();
            url.path_segments_mut().unwrap().pop_if_empty().push(word);
            injector.push(url.to_string());
            Ok(())
        })
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}
