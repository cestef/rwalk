use std::sync::Arc;

use parking_lot::Mutex;
use ptree::TreeItem;
use serde::{Deserialize, Serialize};
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
                &url::Url::parse(&self.data)
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
        write!(f, "/{}", style.paint(&self.data.path))?;
        Ok(())
    }
}
