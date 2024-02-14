use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use colored::Colorize;
use log::{info, warn};
use parking_lot::Mutex;
use ptree::{print_tree, TreeItem};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{cli::Opts, utils::get_emoji_for_status_code_colored, Save};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode<T> {
    pub data: T,
    pub children: Vec<Arc<Mutex<TreeNode<T>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeData {
    pub url: String,
    pub depth: usize,
    pub path: String,
    pub status_code: u16,
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<Arc<Mutex<TreeNode<T>>>>,
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Tree { root: None }
    }

    pub fn insert(
        &mut self,
        data: T,
        parent: Option<Arc<Mutex<TreeNode<T>>>>,
    ) -> Arc<Mutex<TreeNode<T>>> {
        let new_node = Arc::new(Mutex::new(TreeNode {
            data,
            children: Vec::new(),
        }));

        match parent {
            Some(parent) => {
                parent.lock().children.push(new_node.clone());
            }
            None => {
                self.root = Some(new_node.clone());
            }
        }

        new_node
    }

    pub fn get_nodes_at_depth(&self, depth: usize) -> Vec<Arc<Mutex<TreeNode<T>>>> {
        let mut nodes = Vec::new();
        self.get_nodes_at_depth_recursive(&self.root, depth, &mut nodes);
        nodes
    }

    /// Recursively get all nodes at a given depth
    fn get_nodes_at_depth_recursive(
        &self,
        node: &Option<Arc<Mutex<TreeNode<T>>>>,
        depth: usize,
        nodes: &mut Vec<Arc<Mutex<TreeNode<T>>>>,
    ) {
        if let Some(node) = node {
            if depth <= 0 {
                nodes.push(node.clone());
            } else {
                for child in &node.lock().children {
                    self.get_nodes_at_depth_recursive(&Some(child.clone()), depth - 1, nodes);
                }
            }
        }
    }
}

impl TreeItem for TreeNode<String> {
    type Child = TreeNode<String>;
    fn children(&self) -> std::borrow::Cow<[Self::Child]> {
        let mut children = Vec::new();
        for child in &self.children {
            children.push(child.lock().clone());
        }
        std::borrow::Cow::Owned(children)
    }

    fn write_self<W: std::io::Write>(
        &self,
        f: &mut W,
        style: &ptree::Style,
    ) -> std::io::Result<()> {
        write!(
            f,
            "/{}",
            style.paint(
                &url::Url::parse(&self.data.trim_start_matches("/"))
                    .unwrap()
                    .path_segments()
                    .unwrap()
                    .last()
                    .unwrap()
            )
        )?;
        Ok(())
    }
}

impl TreeItem for TreeNode<TreeData> {
    type Child = TreeNode<TreeData>;
    fn children(&self) -> std::borrow::Cow<[Self::Child]> {
        let mut children = Vec::new();
        for child in &self.children {
            children.push(child.lock().clone());
        }
        std::borrow::Cow::Owned(children)
    }

    fn write_self<W: std::io::Write>(
        &self,
        f: &mut W,
        style: &ptree::Style,
    ) -> std::io::Result<()> {
        let emoji = get_emoji_for_status_code_colored(self.data.status_code);
        write!(
            f,
            "{}{} /{}",
            if self.data.status_code == 0 {
                style.paint("🔍".to_string())
            } else {
                style.paint(emoji)
            },
            if self.data.status_code == 0 {
                style.paint("".to_string())
            } else {
                style.paint(format!(" {}", self.data.status_code.to_string().dimmed()))
            },
            style.paint(&self.data.path.trim_start_matches("/"))
        )?;
        Ok(())
    }
}

pub fn from_save(
    opts: &Opts,
    save: &Save,
    depth: Arc<Mutex<usize>>,
    current_indexes: Arc<Mutex<HashMap<String, Vec<usize>>>>,
    words: Vec<String>,
) -> Result<Arc<Mutex<Tree<TreeData>>>> {
    if let Some(root) = &save.tree.clone().lock().root {
        if opts.url.is_some() && root.lock().data.url != opts.url.clone().unwrap() {
            Err(anyhow::anyhow!(
                "The URL of the saved state does not match the URL provided"
            ))
        } else {
            print_tree(&*root.lock())?;
            info!(
                "Found saved state crawled to depth {}",
                (*save.depth.lock() + 1).to_string().blue()
            );

            *depth.lock() = *save.depth.lock();
            if save.wordlist_checksum == { format!("{:x}", md5::compute(words.join("\n"))) } {
                *current_indexes.lock() = save.indexes;
            } else {
                warn!(
                    "Wordlists have changed, starting from scratch at depth {}",
                    (*save.depth.lock() + 1).to_string().yellow()
                );
            }
            Ok(save.tree)
        }
    } else {
        Err(anyhow::anyhow!("No saved state found"))
    }
}
