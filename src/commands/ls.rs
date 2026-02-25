use colored::Colorize;
use serde::Serialize;
use std::collections::BTreeMap;
use walkdir::WalkDir;

use crate::store;

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
}

impl TreeNode {
    fn insert_path(&mut self, components: &[String]) {
        if components.is_empty() {
            return;
        }
        let head = &components[0];
        let child = self.children.entry(head.clone()).or_default();
        child.insert_path(&components[1..]);
    }
}

#[derive(Serialize)]
struct JsonNode {
    name: String,
    kind: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<JsonNode>,
}

pub fn run(subfolder: Option<&str>, json: bool) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let base = match subfolder {
        Some(sub) => store::store_subpath(sub)?,
        None => store::store_dir(),
    };

    if !base.exists() {
        return Err(format!("Path '{}' not found.", base.display()));
    }

    let header = subfolder.unwrap_or("Mind Store");

    let mut root = TreeNode::default();

    for result in WalkDir::new(&base)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
    {
        let entry = result.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
            let rel = path.strip_prefix(&base).unwrap_or(path).with_extension("");
            let components: Vec<String> = rel
                .iter()
                .map(|part| part.to_string_lossy().to_string())
                .collect();
            root.insert_path(&components);
        }
    }

    if json {
        let json_tree = tree_to_json(header, &root);
        let output = serde_json::to_string_pretty(&json_tree)
            .map_err(|e| format!("failed to serialize JSON: {e}"))?;
        println!("{output}");
        return Ok(());
    }

    println!("{}", header.bold());
    if root.children.is_empty() {
        println!("(empty)");
        return Ok(());
    }
    print_tree(&root, "", true);
    Ok(())
}

fn print_tree(node: &TreeNode, prefix: &str, at_root: bool) {
    let count = node.children.len();
    for (idx, (name, child)) in node.children.iter().enumerate() {
        let is_last = idx + 1 == count;
        let branch = if is_last { "└── " } else { "├── " };
        let is_dir = !child.children.is_empty();
        let display_name = if is_dir {
            format!("{}/", name).blue().to_string()
        } else {
            name.to_string()
        };

        if at_root {
            println!("{branch}{display_name}");
        } else {
            println!("{prefix}{branch}{display_name}");
        }

        if is_dir {
            let next_prefix = if at_root {
                if is_last {
                    "    ".to_string()
                } else {
                    "│   ".to_string()
                }
            } else if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };
            print_tree(child, &next_prefix, false);
        }
    }
}

fn tree_to_json(name: &str, node: &TreeNode) -> JsonNode {
    let children: Vec<JsonNode> = node
        .children
        .iter()
        .map(|(child_name, child_node)| tree_to_json(child_name, child_node))
        .collect();
    JsonNode {
        name: name.to_string(),
        kind: if children.is_empty() {
            "entry".to_string()
        } else {
            "folder".to_string()
        },
        children,
    }
}
