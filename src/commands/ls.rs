use chrono::NaiveDate;
use colored::Colorize;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::entry::{normalize_namespace, Entry, Kind, Priority, Status};
use crate::store;

const UNGROUPED: &str = "ungrouped";

#[derive(Default)]
struct TreeNode {
    is_entry: bool,
    children: BTreeMap<String, TreeNode>,
}

impl TreeNode {
    fn insert_path(&mut self, components: &[String]) {
        if components.is_empty() {
            return;
        }
        let child = self.children.entry(components[0].clone()).or_default();
        if components.len() == 1 {
            child.is_entry = true;
            return;
        }
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

#[derive(Clone, Copy)]
enum GroupBy {
    Namespace,
    Kind,
    Priority,
    Status,
    Tags,
}

impl GroupBy {
    fn parse(input: &str) -> Result<Self, String> {
        match input.to_lowercase().as_str() {
            "namespace" => Ok(Self::Namespace),
            "kind" => Ok(Self::Kind),
            "priority" => Ok(Self::Priority),
            "status" => Ok(Self::Status),
            "tags" => Ok(Self::Tags),
            _ => Err(format!(
                "unknown group '{input}', expected one of: namespace, kind, priority, status, tags"
            )),
        }
    }
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

struct StoredEntry {
    slug: String,
    entry: Entry,
}

pub fn run(
    namespace_prefix: Option<&str>,
    group: &str,
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
    let group = GroupBy::parse(group)?;
    let namespace_prefix = namespace_prefix
        .map(normalize_namespace)
        .transpose()?
        .flatten();
    let filters = parse_filters(kind, priority, status, tags, deadline, before, after)?;

    let mut root = TreeNode::default();
    for path in store::list_entry_paths()? {
        let slug = match store::entry_slug(&path) {
            Some(value) => value,
            None => continue,
        };
        let mut entry = store::load_entry(&path)?;
        entry.normalize()?;
        if !matches_filters(&entry, &filters) || !namespace_matches(entry.namespace.as_deref(), namespace_prefix.as_deref()) {
            continue;
        }
        let stored = StoredEntry { slug, entry };
        let groups = groups_for_entry(&stored, group);
        for group_parts in groups {
            let mut components = group_parts;
            components.push(stored.slug.clone());
            root.insert_path(&components);
        }
    }

    if json {
        let output = serde_json::to_string_pretty(&root_children_to_json(&root))
            .map_err(|e| format!("failed to serialize JSON: {e}"))?;
        println!("{output}");
        return Ok(());
    }

    println!("{}", "Mind Store".bold());
    if root.children.is_empty() {
        println!("(empty)");
        return Ok(());
    }
    print_tree(&root, "", true);
    Ok(())
}

fn groups_for_entry(stored: &StoredEntry, group: GroupBy) -> Vec<Vec<String>> {
    match group {
        GroupBy::Namespace => {
            let mut components = namespace_components(stored.entry.namespace.as_deref());
            if components.is_empty() {
                components.push(UNGROUPED.to_string());
            }
            vec![components]
        }
        GroupBy::Kind => vec![vec![stored.entry.kind.to_string()]],
        GroupBy::Priority => vec![vec![stored.entry.priority.to_string()]],
        GroupBy::Status => vec![vec![stored.entry.status.to_string()]],
        GroupBy::Tags => {
            if stored.entry.tags.is_empty() {
                vec![vec![UNGROUPED.to_string()]]
            } else {
                let mut set = BTreeSet::new();
                for tag in &stored.entry.tags {
                    set.insert(tag.to_string());
                }
                set.into_iter().map(|tag| vec![tag]).collect()
            }
        }
    }
}

fn namespace_components(namespace: Option<&str>) -> Vec<String> {
    namespace
        .map(|n| n.split('/').map(|part| part.to_string()).collect())
        .unwrap_or_default()
}

fn namespace_matches(entry_namespace: Option<&str>, prefix: Option<&str>) -> bool {
    match prefix {
        None => true,
        Some(prefix) => match entry_namespace {
            Some(namespace) => namespace == prefix || namespace.starts_with(&format!("{prefix}/")),
            None => false,
        },
    }
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
        let tags: Vec<String> = entry.tags.iter().map(|t| t.to_lowercase()).collect();
        if !filters
            .tags
            .iter()
            .any(|requested| tags.iter().any(|tag| tag == requested))
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

fn sorted_children(node: &TreeNode) -> Vec<(&String, &TreeNode)> {
    let mut children: Vec<(&String, &TreeNode)> = node.children.iter().collect();
    children.sort_by(|(a, _), (b, _)| {
        if a.as_str() == UNGROUPED && b.as_str() != UNGROUPED {
            std::cmp::Ordering::Greater
        } else if a.as_str() != UNGROUPED && b.as_str() == UNGROUPED {
            std::cmp::Ordering::Less
        } else {
            a.cmp(b)
        }
    });
    children
}

fn print_tree(node: &TreeNode, prefix: &str, at_root: bool) {
    let items = render_items(node);
    for (idx, item) in items.iter().enumerate() {
        let is_last = idx + 1 == items.len();
        let branch = if is_last { "└── " } else { "├── " };
        let display_name = if item.as_folder {
            format!("{}/", item.name).blue().to_string()
        } else {
            item.name.to_string()
        };
        if at_root {
            println!("{branch}{display_name}");
        } else {
            println!("{prefix}{branch}{display_name}");
        }
        if item.as_folder {
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
            print_tree(item.child, &next_prefix, false);
        }
    }
}

struct RenderItem<'a> {
    name: &'a str,
    child: &'a TreeNode,
    as_folder: bool,
}

fn render_items<'a>(node: &'a TreeNode) -> Vec<RenderItem<'a>> {
    let mut items = Vec::new();
    for (name, child) in sorted_children(node) {
        if child.is_entry {
            items.push(RenderItem {
                name,
                child,
                as_folder: false,
            });
        }
        if !child.children.is_empty() {
            items.push(RenderItem {
                name,
                child,
                as_folder: true,
            });
        }
    }
    items
}

fn node_children_to_json(node: &TreeNode) -> Vec<JsonNode> {
    let mut out = Vec::new();
    for item in render_items(node) {
        if item.as_folder {
            out.push(JsonNode {
                name: item.name.to_string(),
                kind: "folder".to_string(),
                children: node_children_to_json(item.child),
            });
        } else {
            out.push(JsonNode {
                name: item.name.to_string(),
                kind: "entry".to_string(),
                children: Vec::new(),
            });
        }
    }
    out
}

fn root_children_to_json(root: &TreeNode) -> Vec<JsonNode> {
    node_children_to_json(root)
}

#[cfg(test)]
mod tests {
    use super::{groups_for_entry, namespace_matches, sorted_children, GroupBy, StoredEntry, TreeNode};
    use crate::entry::{Entry, Kind, Priority};

    #[test]
    fn namespace_prefix_match_uses_hierarchy_prefix() {
        assert!(namespace_matches(Some("personal/garage"), Some("personal")));
        assert!(namespace_matches(Some("personal"), Some("personal")));
        assert!(!namespace_matches(Some("work"), Some("personal")));
        assert!(!namespace_matches(None, Some("personal")));
    }

    #[test]
    fn sorted_children_keeps_ungrouped_last() {
        let mut root = TreeNode::default();
        root.insert_path(&["ungrouped".to_string(), "zeta".to_string()]);
        root.insert_path(&["alpha".to_string(), "one".to_string()]);
        root.insert_path(&["beta".to_string(), "two".to_string()]);
        let ordered = sorted_children(&root)
            .into_iter()
            .map(|(name, _)| name.to_string())
            .collect::<Vec<_>>();
        assert_eq!(ordered, vec!["alpha".to_string(), "beta".to_string(), "ungrouped".to_string()]);
    }

    #[test]
    fn tree_supports_same_name_entry_and_folder() {
        let mut root = TreeNode::default();
        root.insert_path(&["test".to_string(), "test".to_string()]);
        root.insert_path(&["test".to_string(), "test".to_string(), "child".to_string()]);
        let level = root.children.get("test").unwrap().children.get("test").unwrap();
        assert!(level.is_entry);
        assert!(!level.children.is_empty());
    }

    #[test]
    fn group_by_tags_creates_multiple_groups() {
        let mut entry = Entry::new(
            "Title".to_string(),
            Kind::Idea,
            Priority::Medium,
            crate::entry::Status::Active,
        );
        entry.tags = vec!["rust".to_string(), "urgent".to_string()];
        let stored = StoredEntry {
            slug: "rework".to_string(),
            entry,
        };
        let groups = groups_for_entry(&stored, GroupBy::Tags);
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn group_by_priority_uses_required_priority() {
        let entry = Entry::new(
            "Title".to_string(),
            Kind::Todo,
            Priority::Medium,
            crate::entry::Status::Active,
        );
        let stored = StoredEntry {
            slug: "task".to_string(),
            entry,
        };
        let groups = groups_for_entry(&stored, GroupBy::Priority);
        assert_eq!(groups, vec![vec!["medium".to_string()]]);

        let mut entry2 = Entry::new(
            "Title".to_string(),
            Kind::Todo,
            Priority::Medium,
            crate::entry::Status::Active,
        );
        entry2.priority = Priority::High;
        let stored2 = StoredEntry {
            slug: "task2".to_string(),
            entry: entry2,
        };
        let groups2 = groups_for_entry(&stored2, GroupBy::Priority);
        assert_eq!(groups2, vec![vec!["high".to_string()]]);
    }

    #[test]
    fn group_by_status_uses_status_value() {
        let mut entry = Entry::new(
            "Title".to_string(),
            Kind::Todo,
            Priority::Medium,
            crate::entry::Status::Active,
        );
        entry.status = crate::entry::Status::Done;
        let stored = StoredEntry {
            slug: "task".to_string(),
            entry,
        };
        let groups = groups_for_entry(&stored, GroupBy::Status);
        assert_eq!(groups, vec![vec!["done".to_string()]]);
    }
}
