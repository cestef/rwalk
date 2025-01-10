use crate::wordlist::{transformation::WordlistTransformation, Wordlist};

struct Lowercase;

impl WordlistTransformation for Lowercase {
    fn transform(&self, wordlist: &mut Wordlist) {
        wordlist.0 = wordlist.0.iter().map(|s| s.to_lowercase()).collect();
    }
    fn name(&self) -> &str {
        "lowercase"
    }
    fn aliases(&self) -> &[&str] {
        &["lower"]
    }
}
