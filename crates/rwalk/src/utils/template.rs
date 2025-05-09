use cowstr::CowStr;

use crate::wordlist::Wordlist;

pub fn find_keys(text: &str, wordlists: &[Wordlist]) -> Vec<(usize, CowStr)> {
    wordlists
        .iter()
        .flat_map(|wl| {
            text.match_indices(wl.key.as_str())
                .map(move |(pos, _)| (pos, wl.key.clone()))
        })
        .collect::<Vec<_>>()
}
