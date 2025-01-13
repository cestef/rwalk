use dashmap::DashMap as HashMap;
use url::Url;

use crate::worker::utils::RwalkResponse;

#[derive(Default)]
struct Node {
    children: HashMap<String, Node>,
    is_endpoint: bool,
}

pub fn display_url_tree(urls: &HashMap<String, RwalkResponse>) {
    let mut root = Node::default();

    // Build the tree structure
    for e in urls.iter() {
        let url = e.key();
        if let Ok(parsed_url) = Url::parse(url) {
            let path = parsed_url.path();
            let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

            insert_path(&mut root, &components);
        }
    }

    // Display the tree
    print_tree(&root, 0, "");
}

fn insert_path(node: &mut Node, components: &[&str]) {
    if components.is_empty() {
        node.is_endpoint = true;
        return;
    }

    let component = components[0];
    let mut child = node.children.entry(component.to_string()).or_default();

    insert_path(&mut child, &components[1..]);
}

fn print_tree(node: &Node, depth: usize, prefix: &str) {
    for (i, child) in node.children.iter().enumerate() {
        let name = child.key();
        let is_last = i == node.children.len() - 1;

        // Print current node
        let branch = if is_last { "└── " } else { "├── " };
        println!("{}{}/{}", prefix, branch, name);

        // Prepare prefix for children
        let child_prefix = if is_last {
            format!("{}   ", prefix)
        } else {
            format!("{}│  ", prefix)
        };

        // Recursively print children
        print_tree(&child, depth + 1, &child_prefix);
    }
}
