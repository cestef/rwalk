use std::{collections::HashMap, sync::Arc};

use crate::cli::opts::Opts;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use super::tree::{Tree, TreeData};

pub enum Mode {
    Recursive,
    Classic,
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

#[derive(Serialize, Deserialize)]
pub struct Save {
    pub tree: Arc<Mutex<Tree<TreeData>>>,
    pub depth: Arc<Mutex<usize>>,
    pub wordlist_checksum: String,
    pub indexes: HashMap<String, Vec<usize>>,
    pub opts: Opts,
}
