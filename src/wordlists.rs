use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use crate::ARGS;

const COMMENT_PREFIXES: [&str; 2] = ["#", "//"];
pub struct WordlistParser {
    pub wordlists: Vec<PathBuf>,
    pub contents: Vec<String>,
}

impl WordlistParser {
    pub fn new(wordlists: Vec<PathBuf>) -> Self {
        Self {
            wordlists,
            contents: vec![],
        }
    }

    pub fn read(&mut self) -> Result<()> {
        let contents: Vec<String> = self
            .wordlists
            .iter()
            .map(|path| {
                let file =
                    File::open(path).with_context(|| format!("Failed to open file {:?}", &path))?;
                let mut reader = BufReader::new(&file);
                let mut contents = String::new();
                reader
                    .read_to_string(&mut contents)
                    .with_context(|| format!("Failed to read file {:?}", &file))?;
                Ok(contents)
            })
            .collect::<Result<Vec<String>>>()?;
        self.contents = contents.clone();
        Ok(())
    }

    pub fn merge(&mut self) -> Result<()> {
        let words = self
            .contents
            .join("\n")
            .split("\n")
            .map(|s| {
                if ARGS.case_insensitive {
                    s.to_lowercase()
                } else {
                    s.to_string()
                }
            })
            .filter(|s| !s.is_empty())
            .filter(|s| !COMMENT_PREFIXES.iter().any(|p| s.starts_with(p)))
            .collect::<Vec<String>>();
        self.contents = words.clone();
        Ok(())
    }
}
