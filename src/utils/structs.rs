use std::{collections::HashMap, sync::Arc};

use crate::cli::opts::Opts;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use super::tree::{Tree, TreeData};

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum Mode {
    Recursive,
    Classic,
}

#[derive(Clone, Debug)]
pub struct FuzzMatch {
    pub content: String,
    pub start: usize,
    pub end: usize,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Recursive => "Recursive".to_string(),
            Mode::Classic => "Classic".to_string(),
        }
    }
}

impl From<&str> for Mode {
    fn from(s: &str) -> Self {
        match s {
            "recursive" | "recursion" | "r" => Mode::Recursive,
            "classic" | "c" => Mode::Classic,
            _ => Mode::Recursive,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Save {
    pub tree: Arc<Mutex<Tree<TreeData>>>,
    pub depth: Arc<Mutex<usize>>,
    pub wordlist_checksum: String,
    pub indexes: HashMap<String, Vec<usize>>,
    pub opts: Opts,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_to_string() {
        assert_eq!(Mode::Recursive.to_string(), "Recursive");
        assert_eq!(Mode::Classic.to_string(), "Classic");
    }

    #[test]
    fn test_mode_from_str() {
        assert_eq!(Mode::from("recursive"), Mode::Recursive);
        assert_eq!(Mode::from("recursion"), Mode::Recursive);
        assert_eq!(Mode::from("r"), Mode::Recursive);
        assert_eq!(Mode::from("classic"), Mode::Classic);
        assert_eq!(Mode::from("c"), Mode::Classic);
        assert_eq!(Mode::from("invalid"), Mode::Recursive);
    }
}
