use colored::Colorize;
use chrono::NaiveDate;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use walkdir::WalkDir;

use crate::entry::{Entry, Kind, Priority, Status};
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

struct LsFilters {
    kind: Option<Kind>,
    priority: Option<Priority>,
    status: Option<Status>,
    tags: Vec<String>,
    deadline: Option<NaiveDate>,
    before: Option<NaiveDate>,
    after: Option<NaiveDate>,
}

pub fn run(
    subfolder: Option<&str>,
    json: bool,
    kind: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
    tags: &[String],
    deadline: Option<&str>,
    before: Option<&str>,
    after: Option<&str>,
) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;
    let filters = parse_filters(kind, priority, status, tags, deadline, before, after)?;

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
            let contents =
                fs::read_to_string(path).map_err(|e| format!("failed to read entry file: {e}"))?;
            let parsed: Entry = Entry::from_toml(&contents)
                .map_err(|e| format!("failed to parse entry '{}': {e}", path.display()))?;
            if !matches_filters(&parsed, &filters) {
                continue;
            }

            let rel = path.strip_prefix(&base).unwrap_or(path).with_extension("");
            let components: Vec<String> = rel
                .iter()
                .map(|part| part.to_string_lossy().to_string())
                .collect();
            root.insert_path(&components);
        }
    }

    if json {
        let json_nodes = root_children_to_json(&root);
        let output = serde_json::to_string_pretty(&json_nodes)
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

fn parse_filters(
    kind: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
    tags: &[String],
    deadline: Option<&str>,
    before: Option<&str>,
    after: Option<&str>,
) -> Result<LsFilters, String> {
    let kind = kind.map(str::parse).transpose()?;
    let priority = priority.map(str::parse).transpose()?;
    let status = status.map(str::parse).transpose()?;
    let deadline = parse_date(deadline, "--deadline")?;
    let before = parse_date(before, "--before")?;
    let after = parse_date(after, "--after")?;

    Ok(LsFilters {
        kind,
        priority,
        status,
        tags: tags.iter().map(|t| t.to_lowercase()).collect(),
        deadline,
        before,
        after,
    })
}

fn parse_date(input: Option<&str>, label: &str) -> Result<Option<NaiveDate>, String> {
    input
        .map(|value| {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|e| format!("invalid {label} date format: {e}"))
        })
        .transpose()
}

fn matches_filters(entry: &Entry, filters: &LsFilters) -> bool {
    if let Some(kind) = &filters.kind {
        if &entry.kind != kind {
            return false;
        }
    }

    if let Some(priority) = &filters.priority {
        if &entry.priority != priority {
            return false;
        }
    }
    if let Some(status) = &filters.status {
        if &entry.status != status {
            return false;
        }
    }

    if !filters.tags.is_empty() {
        let entry_tags: Vec<String> = entry.tags.iter().map(|t| t.to_lowercase()).collect();
        if !filters
            .tags
            .iter()
            .any(|requested| entry_tags.iter().any(|tag| tag == requested))
        {
            return false;
        }
    }

    if let Some(deadline) = filters.deadline {
        if entry.deadline != Some(deadline) {
            return false;
        }
    }

    if let Some(before) = filters.before {
        match entry.deadline {
            Some(deadline) if deadline < before => {}
            _ => return false,
        }
    }

    if let Some(after) = filters.after {
        match entry.deadline {
            Some(deadline) if deadline > after => {}
            _ => return false,
        }
    }

    true
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

fn root_children_to_json(root: &TreeNode) -> Vec<JsonNode> {
    root.children
        .iter()
        .map(|(child_name, child_node)| tree_to_json(child_name, child_node))
        .collect()
}
