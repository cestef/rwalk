use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use colored::Colorize;
use log::{info, warn};
use parking_lot::Mutex;
use ptree::{print_tree, TreeItem};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    cli::opts::Opts,
    runner::wordlists::{compute_checksum, ParsedWordlist},
    utils::get_emoji_for_status_code_colored,
    Save,
};
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
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<Arc<Mutex<TreeNode<T>>>>,
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Tree { root: None }
    }
    /// Insert a new data into the tree, at the root if no parent provided.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to insert
    /// * `parent` - The parent node, or `None` to insert at the root
    ///
    /// # Returns
    ///
    /// A new `Arc<Mutex<TreeNode<T>>>` containing the newly inserted node.
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

    /// Recursively get all nodes at a given depth
    ///
    /// # Arguments
    ///
    /// * `depth` - The depth to get nodes at
    ///
    /// # Returns
    ///
    /// A vector of all nodes at the given depth
    ///
    pub fn get_nodes_at_depth(&self, depth: usize) -> Vec<Arc<Mutex<TreeNode<T>>>> {
        let mut nodes = Vec::new();
        Self::get_nodes_at_depth_recursive(&self.root, depth, &mut nodes);
        nodes
    }

    /// Recursively get all nodes at a given depth
    ///
    /// # Arguments
    ///
    /// * `depth` - The depth to get nodes at
    ///
    /// # Returns
    ///
    /// A vector of all nodes at the given depth
    ///
    /// # Notes
    ///
    /// This function is a helper function for `get_nodes_at_depth`
    fn get_nodes_at_depth_recursive(
        node: &Option<Arc<Mutex<TreeNode<T>>>>,
        depth: usize,
        nodes: &mut Vec<Arc<Mutex<TreeNode<T>>>>,
    ) {
        if let Some(node) = node {
            if depth == 0 {
                nodes.push(node.clone());
            } else {
                for child in &node.lock().children {
                    Self::get_nodes_at_depth_recursive(&Some(child.clone()), depth - 1, nodes);
                }
            }
        }
    }

    /// Insert a vector of data into the tree
    ///
    /// # Arguments
    ///
    /// * `datas` - The data to insert
    ///
    /// # Notes
    ///
    /// This function will insert the data at the root of the tree
    ///
    pub fn insert_datas(&mut self, datas: Vec<T>) {
        // Insert nodes into the root
        let mut previous_node: Option<Arc<Mutex<TreeNode<T>>>> = self.root.clone();
        for data in datas {
            previous_node = Some(self.insert(data, previous_node));
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
                &url::Url::parse(self.data.trim_start_matches('/'))
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
            style.paint(&self.data.path.trim_start_matches('/'))
        )?;
        Ok(())
    }
}

/// Create a new tree from a save
pub fn from_save(
    opts: &Opts,
    save: &Save,
    depth: Arc<Mutex<usize>>,
    current_indexes: Arc<Mutex<HashMap<String, Vec<usize>>>>,
    words: HashMap<String, ParsedWordlist>,
) -> Result<Arc<Mutex<Tree<TreeData>>>> {
    if let Some(root) = &save.tree.clone().lock().root {
        if opts.url.is_some() && root.lock().data.url != opts.url.clone().unwrap() {
            Err(anyhow::anyhow!(
                "The URL of the saved state does not match the URL provided"
            ))
        } else {
            info!(
                "Found saved state crawled to depth {}",
                (*save.depth.lock() + 1).to_string().bold()
            );
            print_tree(&*root.lock())?;
            *depth.lock() = *save.depth.lock();
            if save.wordlist_checksum == { compute_checksum(&words) } {
                *current_indexes.lock() = save.indexes.clone();
            } else {
                warn!(
                    "Wordlists have changed, starting from scratch at depth {}",
                    (*save.depth.lock() + 1).to_string().yellow()
                );
            }
            Ok(save.tree.clone())
        }
    } else {
        Err(anyhow::anyhow!("No saved state found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_insert() {
        let mut tree = Tree::new();
        let node1 = tree.insert("node1".to_string(), None);
        assert!(tree.root.is_some());
        let tree_root = (tree.root.as_ref().unwrap().lock()).clone();
        assert_eq!(tree_root.data, "node1".to_string());
        assert_eq!(tree_root.children.len(), 0);
        let _ = tree.insert("node2".to_string(), Some(node1.clone()));
        let tree_root = (tree.root.as_ref().unwrap().lock()).clone();
        assert_eq!(tree_root.children.len(), 1);
        assert_eq!(tree_root.children[0].lock().data, "node2".to_string());
    }

    #[test]
    fn test_tree_get_nodes_at_depth() {
        let mut tree = Tree::new();
        let node1 = tree.insert("node1".to_string(), None);
        let node2 = tree.insert("node2".to_string(), Some(node1.clone()));
        let _node3 = tree.insert("node3".to_string(), Some(node1.clone()));
        let _node4 = tree.insert("node4".to_string(), Some(node2.clone()));
        let _node5 = tree.insert("node5".to_string(), Some(node2.clone()));

        let nodes = tree.get_nodes_at_depth(0);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].lock().data, "node1".to_string());

        let nodes = tree.get_nodes_at_depth(1);
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].lock().data, "node2".to_string());
        assert_eq!(nodes[1].lock().data, "node3".to_string());

        let nodes = tree.get_nodes_at_depth(2);
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].lock().data, "node4".to_string());
        assert_eq!(nodes[1].lock().data, "node5".to_string());
    }

    #[test]
    fn test_tree_insert_datas() {
        let mut tree = Tree::new();
        tree.insert_datas(vec!["node1".to_string(), "node2".to_string()]);
        let tree_root = (tree.root.as_ref().unwrap().lock()).clone();
        assert_eq!(tree_root.data, "node1".to_string());
        assert_eq!(tree_root.children.len(), 1);
        assert_eq!(tree_root.children[0].lock().data, "node2".to_string());
    }

    #[test]
    fn test_tree_item_write_self() {
        let node = TreeNode {
            data: "http://example.com/test".to_string(),
            children: vec![],
        };
        let mut buffer = Vec::new();
        node.write_self(&mut buffer, &ptree::Style::default())
            .unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "/test");
    }

    #[test]
    fn test_tree_item_write_self_with_emoji() {
        let node = TreeNode {
            data: TreeData {
                url: "http://example.com/test".to_string(),
                depth: 0,
                path: "/test".to_string(),
                status_code: 200,
                extra: Value::Null,
                is_dir: false,
            },
            children: vec![],
        };
        let mut buffer = Vec::new();
        node.write_self(&mut buffer, &ptree::Style::default())
            .unwrap();
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            "✓ 200 /test".to_string()
        );
    }
}
