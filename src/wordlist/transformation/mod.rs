use super::Wordlist;

pub mod lower;

pub trait WordlistTransformation {
    fn transform(&self, wordlist: &mut Wordlist);
    fn aliases(&self) -> &[&str] {
        &[]
    }
    fn name(&self) -> &str;
}
