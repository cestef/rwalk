use std::{borrow::Cow, io};

use async_recursion::async_recursion;
use indicatif::ProgressBar;
use ptree::TreeItem;
use url::Url;

use crate::manager;

#[derive(Debug, Clone)]
pub struct PathTree {
    pub name: String,
    pub children: Vec<PathTree>,
}

impl TreeItem for PathTree {
    type Child = Self;
    fn write_self<W: io::Write>(&self, f: &mut W, _style: &ptree::Style) -> io::Result<()> {
        write!(f, "/{}", self.name.split("/").last().unwrap())
    }
    fn children(&self) -> Cow<[Self::Child]> {
        Cow::from(&self.children[..])
    }
}

pub struct TreeTraverser {
    host: Url,
    words: Vec<String>,
    threads: usize,
    progress: ProgressBar,
    pub tree: PathTree,
    depth: u8,
}

impl TreeTraverser {
    pub fn new(
        host: Url,
        words: Vec<String>,
        threads: usize,
        progress: ProgressBar,
        tree: PathTree,
        depth: u8,
    ) -> Self {
        Self {
            host,
            words,
            threads,
            progress,
            tree,
            depth,
        }
    }

    #[async_recursion]
    pub async fn traverse(&mut self) {
        if self.depth == 0 {
            return;
        }

        for child in &mut self.tree.children {
            let mut new_url = self.host.clone();
            new_url.set_path(&child.name);
            self.progress.set_position(0);
            self.progress
                .set_message(format!("🔎 Crawling {}", new_url));
            let manager = manager::CrawlerManager::new(
                new_url,
                self.words.clone(),
                self.threads,
                self.progress.clone(),
            );
            child.children = manager
                .run()
                .await
                .unwrap()
                .iter()
                .map(|urls| PathTree {
                    name: Url::parse(&urls[0])
                        .unwrap()
                        .path()
                        .trim_end_matches("/")
                        .to_string(),
                    children: Vec::new(),
                })
                .collect::<Vec<PathTree>>();

            let mut traverser = TreeTraverser::new(
                self.host.clone(),
                self.words.clone(),
                self.threads,
                self.progress.clone(),
                child.clone(),
                self.depth - 1,
            );
            traverser.traverse().await;
        }
    }
}

pub fn tree_to_vec(tree: &PathTree, out: &mut Vec<String>) {
    for child in &tree.children {
        out.push(child.name.clone());
        tree_to_vec(child, out);
    }
}
