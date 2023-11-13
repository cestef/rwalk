use std::{
    borrow::Cow,
    io,
    sync::{atomic::AtomicU8, Arc},
};

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
    init: bool,
    pub tree: PathTree,
}

impl TreeTraverser {
    pub fn new(
        host: Url,
        words: Vec<String>,
        threads: usize,
        progress: ProgressBar,
        tree: PathTree,
    ) -> Self {
        Self {
            host,
            words,
            threads,
            progress,
            tree,
            init: false,
        }
    }

    #[async_recursion]
    pub async fn traverse(&mut self, depth: Arc<AtomicU8>) {
        let loaded = depth.load(std::sync::atomic::Ordering::Relaxed);
        if loaded == 0 {
            return;
        }

        if self.tree.children.len() == 0 && !self.init {
            let mut new_url = self.host.clone();
            new_url.set_path(&self.tree.name);
            self.progress.set_position(0);
            self.progress
                .set_message(format!("🔎 Crawling d=init {}", new_url));
            let manager = manager::CrawlerManager::new(
                new_url,
                self.words.clone(),
                self.threads,
                self.progress.clone(),
            );
            self.tree.children = manager
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

            self.init = true;
        }

        for child in &mut self.tree.children {
            let mut new_url = self.host.clone();
            new_url.set_path(&child.name);
            self.progress.set_position(0);
            self.progress
                .set_message(format!("🔎 Crawling d={} {}", loaded, new_url));
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
            );
            let depth_clone = depth.clone();
            depth_clone.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            traverser.traverse(depth_clone).await;
        }
    }
}

pub fn tree_to_vec(tree: &PathTree, out: &mut Vec<String>) {
    for child in &tree.children {
        out.push(child.name.clone());
        tree_to_vec(child, out);
    }
}
