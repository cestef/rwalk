use dashmap::DashMap;
use ptree::{Style, TreeItem};
use std::borrow::Cow;
use std::collections::HashMap;
use url::Url;

use crate::utils::format::display_status_code;
use crate::worker::utils::RwalkResponse;

#[derive(Default, Clone)]
struct Node {
    name: String,
    status: u16,
    children: HashMap<String, Node>,
    is_endpoint: bool,
}

impl Node {
    fn simplify(&mut self) {
        for child in self.children.values_mut() {
            child.simplify();
        }

        let mut i = 0;
        while i < self.children.len() {
            let mut keys_to_remove = Vec::new();
            let mut nodes_to_add = HashMap::new();

            // Find children with only one child that aren't endpoints
            for (key, child) in &self.children {
                if child.children.len() == 1 && !child.is_endpoint {
                    let (grandchild_key, grandchild) = child.children.iter().next().unwrap();
                    let new_key = format!("{}{}", key, grandchild_key);

                    keys_to_remove.push(key.clone());
                    nodes_to_add.insert(new_key, grandchild.clone());
                }
            }

            if keys_to_remove.is_empty() {
                break;
            }

            for key in keys_to_remove {
                self.children.remove(&key);
            }

            for (key, node) in nodes_to_add {
                self.children.insert(key, node);
            }

            i += 1;
        }
    }
}

impl TreeItem for Node {
    type Child = Node;

    fn children(&self) -> Cow<[Self::Child]> {
        let mut children: Vec<Node> = self
            .children
            .iter()
            .map(|(key, value)| {
                let mut child = value.clone();
                child.name = key.clone();
                child
            })
            .collect();

        children.sort_by(|a, b| a.name.cmp(&b.name));
        Cow::Owned(children)
    }

    fn write_self<W: std::io::Write>(&self, f: &mut W, style: &Style) -> std::io::Result<()> {
        if !self.name.is_empty() {
            write!(
                f,
                "{} {}",
                display_status_code(self.status),
                style.paint(&self.name)
            )
        } else {
            Ok(())
        }
    }
}

pub fn display_url_tree(base: &Url, urls: &DashMap<String, RwalkResponse>) {
    let mut root = Node {
        name: String::new(),
        ..Default::default()
    };

    for entry in urls.iter() {
        let url = entry.key();
        if let Ok(parsed_url) = Url::parse(url) {
            let path = parsed_url.path();

            let components = path
                .split('/')
                .filter(|s| !s.is_empty())
                .map(|comp| format!("/{}", comp))
                .collect::<Vec<String>>();

            insert_path(&mut root, &components, entry.value());
        }
    }

    root.simplify();
    if root.children.is_empty() {
        return;
    }
    print!("\n{}://{}", base.scheme(), base.host_str().unwrap());

    ptree::print_tree(&root).unwrap();
}

fn insert_path(node: &mut Node, components: &[String], response: &RwalkResponse) {
    if components.is_empty() {
        node.is_endpoint = true;
        node.status = response.status as u16;
        return;
    }

    let component = components[0].clone();
    let child = node
        .children
        .entry(component.clone())
        .or_insert_with(|| Node {
            name: component,
            status: response.status as u16,
            ..Default::default()
        });

    insert_path(child, &components[1..], response);
}
