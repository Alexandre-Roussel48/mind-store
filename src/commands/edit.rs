use chrono::NaiveDate;
use dialoguer::{Editor, Input, Select};
use std::fs;

use crate::entry::{normalize_namespace, normalize_tags, Entry, Kind, Priority, Status};
use crate::{extensions, git, store};

pub fn run(
    identifier: &str,
    title: Option<&str>,
    namespace: Option<&str>,
    kind: Option<&str>,
    description: Option<&str>,
    append_description: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
    deadline: Option<&str>,
    tags: Option<&str>,
) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let (path, expected_namespace, slug) = store::resolve_identifier(identifier)?;
    if !path.exists() {
        return Err(format!("Entry '{slug}' not found."));
    }

    let contents = fs::read_to_string(&path).map_err(|e| format!("failed to read entry: {e}"))?;
    let mut entry =
        Entry::from_toml(&contents).map_err(|e| format!("failed to parse entry: {e}"))?;

    let has_flags =
        title.is_some()
            || namespace.is_some()
            || kind.is_some()
            || description.is_some()
            || append_description.is_some()
            || priority.is_some()
            || status.is_some()
            || deadline.is_some()
            || tags.is_some();

    if let Some(ns) = expected_namespace {
        if entry.namespace.as_deref() != Some(ns.as_str()) {
            return Err(format!(
                "Entry '{slug}' exists but is in namespace '{}' (requested '{}').",
                entry.namespace.unwrap_or_else(|| "ungrouped".to_string()),
                ns
            ));
        }
    }

    if has_flags {
        apply_flags(
            &mut entry,
            title,
            namespace,
            kind,
            description,
            append_description,
            priority,
            status,
            deadline,
            tags,
        )?;
    } else {
        apply_interactive(&mut entry)?;
    }

    entry.normalize()?;
    entry.touch();
    let toml = entry
        .to_toml()
        .map_err(|e| format!("serialization error: {e}"))?;
    fs::write(&path, &toml).map_err(|e| format!("failed to write entry: {e}"))?;

    git::add_and_commit(&format!("mind: edit {slug}"))?;
    extensions::run_event("edit", &slug, Some(&entry));
    println!("Updated entry '{slug}'.");
    Ok(())
}

fn apply_flags(
    entry: &mut Entry,
    title: Option<&str>,
    namespace: Option<&str>,
    kind: Option<&str>,
    description: Option<&str>,
    append_description: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
    deadline: Option<&str>,
    tags: Option<&str>,
) -> Result<(), String> {
    if description.is_some() && append_description.is_some() {
        return Err("use either --description or --append-description, not both".to_string());
    }

    if let Some(v) = title {
        if v.trim().is_empty() {
            return Err("title cannot be empty".to_string());
        }
        entry.title = v.trim().to_string();
    }
    if let Some(v) = namespace {
        if v == "none" {
            entry.namespace = None;
        } else {
            entry.namespace = normalize_namespace(v)?;
        }
    }
    if let Some(k) = kind {
        entry.kind = k.parse()?;
    }
    if let Some(d) = description {
        entry.description = Some(d.to_string()).filter(|s| !s.trim().is_empty());
    } else if let Some(d) = append_description {
        if entry.description.as_deref().unwrap_or_default().is_empty() {
            entry.description = Some(d.to_string());
        } else {
            entry.description = Some(format!(
                "{}\n{}",
                entry.description.as_deref().unwrap_or_default(),
                d
            ));
        }
    }
    if let Some(p) = priority {
        entry.priority = p.parse()?;
    }
    if let Some(s) = status {
        entry.status = s.parse()?;
    }
    if let Some(d) = deadline {
        if d == "none" {
            entry.deadline = None;
        } else {
            entry.deadline = Some(
                NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .map_err(|e| format!("invalid date format: {e}"))?,
            );
        }
    }
    if let Some(t) = tags {
        if t == "none" {
            entry.tags = Vec::new();
        } else {
            let parsed: Vec<String> = t.split(',').map(|s| s.to_string()).collect();
            entry.tags = normalize_tags(&parsed);
        }
    }
    Ok(())
}

fn apply_interactive(entry: &mut Entry) -> Result<(), String> {
    let title_input: String = Input::new()
        .with_prompt("Title")
        .with_initial_text(entry.title.clone())
        .interact_text()
        .map_err(|e| e.to_string())?;
    if title_input.trim().is_empty() {
        return Err("title cannot be empty".to_string());
    }
    entry.title = title_input.trim().to_string();

    let namespace_default = entry.namespace.clone().unwrap_or_default();
    let namespace_input: String = Input::new()
        .with_prompt("Namespace (leave empty for ungrouped)")
        .with_initial_text(namespace_default)
        .allow_empty(true)
        .interact_text()
        .map_err(|e| e.to_string())?;
    entry.namespace = normalize_namespace(&namespace_input)?;

    let kind_default = Kind::VARIANTS
        .iter()
        .position(|&v| v == entry.kind.to_string().as_str())
        .unwrap_or(0);
    let kind_idx = Select::new()
        .with_prompt("Kind")
        .items(Kind::VARIANTS)
        .default(kind_default)
        .interact()
        .map_err(|e| e.to_string())?;
    entry.kind = Kind::VARIANTS[kind_idx].parse().unwrap();

    let text = Editor::new()
        .require_save(true)
        .edit(entry.description.as_deref().unwrap_or(""))
        .map_err(|e| e.to_string())?;
    if let Some(d) = text {
        entry.description = Some(d.trim().to_string()).filter(|s| !s.is_empty());
    }

    let current_priority = entry.priority.to_string();
    let priority_default = Priority::VARIANTS
        .iter()
        .position(|&v| v == current_priority)
        .unwrap_or(1);
    let priority_idx = Select::new()
        .with_prompt("Priority")
        .items(Priority::VARIANTS)
        .default(priority_default)
        .interact()
        .map_err(|e| e.to_string())?;
    entry.priority = Priority::VARIANTS[priority_idx].parse().unwrap();

    let current_status = entry.status.to_string();
    let status_default = Status::VARIANTS
        .iter()
        .position(|&v| v == current_status)
        .unwrap_or(0);
    let status_idx = Select::new()
        .with_prompt("Status")
        .items(Status::VARIANTS)
        .default(status_default)
        .interact()
        .map_err(|e| e.to_string())?;
    entry.status = Status::VARIANTS[status_idx].parse().unwrap();

    let deadline_default = entry
        .deadline
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    let deadline_str: String = Input::new()
        .with_prompt("Deadline (YYYY-MM-DD, clear to remove)")
        .with_initial_text(deadline_default)
        .allow_empty(true)
        .interact_text()
        .map_err(|e| e.to_string())?;
    entry.deadline = if deadline_str.is_empty() {
        None
    } else {
        Some(
            NaiveDate::parse_from_str(&deadline_str, "%Y-%m-%d")
                .map_err(|e| format!("invalid date format: {e}"))?,
        )
    };

    let tags_default = entry.tags.join(", ");
    let tags_str: String = Input::new()
        .with_prompt("Tags (comma-separated, clear to remove)")
        .with_initial_text(tags_default)
        .allow_empty(true)
        .interact_text()
        .map_err(|e| e.to_string())?;
    entry.tags = if tags_str.is_empty() {
        Vec::new()
    } else {
        let parsed: Vec<String> = tags_str.split(',').map(|t| t.to_string()).collect();
        normalize_tags(&parsed)
    };

    Ok(())
}
