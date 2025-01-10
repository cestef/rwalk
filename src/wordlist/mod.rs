use crate::Result;
use crossbeam::deque::Injector;

use papaya::HashSet;
use transformation::Transformer;

pub mod filters;
pub mod processor;
pub mod transformation;

pub struct Wordlist(Vec<String>);

impl Wordlist {
    pub async fn from_path(path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let words = content.lines().map(|s| s.to_string()).collect::<Vec<_>>();
        Ok(Self(words))
    }

    pub fn transform(&mut self, transformer: &Transformer<String>) {
        for word in &mut self.0 {
            transformer.apply(word);
        }
    }

    pub fn dedup(&mut self) {
        let seen = HashSet::new();
        self.0.retain(|word| seen.pin().insert(word.clone()));
    }

    pub fn inject(&self, injector: &Injector<String>) {
        for word in &self.0 {
            injector.push(word.clone());
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
