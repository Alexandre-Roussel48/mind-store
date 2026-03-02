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
    priority: Option<&str>,
    status: Option<&str>,
    deadline: Option<&str>,
    tags: Option<&str>,
) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let (compat_namespace, slug) = store::parse_namespace_and_slug(identifier)?;
    let path = store::entry_path(&slug)?;
    if path.exists() {
        return Err(format!(
            "Entry '{slug}' already exists. Use `mind edit {slug}` instead."
        ));
    }

    let effective_namespace = match namespace {
        Some(value) => normalize_namespace(value)?,
        None => compat_namespace,
    };

    let non_interactive = kind.is_some() && priority.is_some();

    let entry = if non_interactive {
        build_from_flags(
            &slug,
            title,
            effective_namespace,
            kind.unwrap(),
            description,
            priority.unwrap(),
            status,
            deadline,
            tags,
        )?
    } else {
        build_interactive(
            &slug,
            title,
            effective_namespace,
            kind,
            description,
            priority,
            status,
            deadline,
            tags,
        )?
    };

    let toml = entry
        .to_toml()
        .map_err(|e| format!("serialization error: {e}"))?;

    fs::write(&path, &toml).map_err(|e| format!("failed to write entry: {e}"))?;

    git::add_and_commit(&format!("mind: add {slug}"))?;
    extensions::run_event("insert", &slug, Some(&entry));
    println!("Inserted entry '{slug}'.");
    Ok(())
}

fn build_from_flags(
    slug: &str,
    title: Option<&str>,
    namespace: Option<String>,
    kind: &str,
    description: Option<&str>,
    priority: &str,
    status: Option<&str>,
    deadline: Option<&str>,
    tags: Option<&str>,
) -> Result<Entry, String> {
    let kind: Kind = kind.parse()?;
    let priority: Priority = priority.parse()?;
    let status: Status = status.map(str::parse).transpose()?.unwrap_or(Status::Active);
    let mut entry = Entry::new(
        title
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| slug.to_string()),
        kind,
        priority,
        status,
    );
    entry.namespace = namespace;
    entry.description = description.map(|d| d.to_string()).filter(|d| !d.trim().is_empty());
    if let Some(d) = deadline {
        entry.deadline = Some(
            NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|e| format!("invalid date format: {e}"))?,
        );
    }

    if let Some(t) = tags {
        let parsed: Vec<String> = t.split(',').map(|s| s.to_string()).collect();
        entry.tags = normalize_tags(&parsed);
    }

    entry.normalize()?;
    Ok(entry)
}

fn build_interactive(
    slug: &str,
    title_flag: Option<&str>,
    namespace: Option<String>,
    kind_flag: Option<&str>,
    desc_flag: Option<&str>,
    priority_flag: Option<&str>,
    status_flag: Option<&str>,
    deadline_flag: Option<&str>,
    tags_flag: Option<&str>,
) -> Result<Entry, String> {
    let title = match title_flag {
        Some(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => {
            let entered: String = Input::new()
                .with_prompt("Title")
                .default(slug.to_string())
                .interact_text()
                .map_err(|e| e.to_string())?;
            entered
        }
    };

    let kind: Kind = match kind_flag {
        Some(k) => k.parse()?,
        None => {
            let idx = Select::new()
                .with_prompt("Kind")
                .items(Kind::VARIANTS)
                .default(0)
                .interact()
                .map_err(|e| e.to_string())?;
            Kind::VARIANTS[idx].parse().unwrap()
        }
    };

    let description = match desc_flag {
        Some(d) if !d.trim().is_empty() => Some(d.trim().to_string()),
        None => {
            let text = Editor::new()
                .require_save(true)
                .edit("Enter description (optional, save and close editor when done)")
                .map_err(|e| e.to_string())?;
            match text {
                Some(d) if !d.trim().is_empty() => Some(d.trim().to_string()),
                _ => None,
            }
        }
        _ => None,
    };

    let namespace = match namespace {
        Some(existing) => Some(existing),
        None => {
            let namespace_input: String = Input::new()
                .with_prompt("Namespace (leave empty for ungrouped)")
                .allow_empty(true)
                .interact_text()
                .map_err(|e| e.to_string())?;
            normalize_namespace(&namespace_input)?
        }
    };

    let priority: Priority = match priority_flag {
        Some(p) => p.parse()?,
        None => {
            let idx = Select::new()
                .with_prompt("Priority")
                .items(Priority::VARIANTS)
                .default(1)
                .interact()
                .map_err(|e| e.to_string())?;
            Priority::VARIANTS[idx].parse().unwrap()
        }
    };

    let status: Status = match status_flag {
        Some(s) => s.parse()?,
        None => {
            let idx = Select::new()
                .with_prompt("Status")
                .items(Status::VARIANTS)
                .default(0)
                .interact()
                .map_err(|e| e.to_string())?;
            Status::VARIANTS[idx].parse().unwrap()
        }
    };
    let mut entry = Entry::new(title, kind, priority, status);
    entry.namespace = namespace;
    entry.description = description;

    match deadline_flag {
        Some(d) => {
            entry.deadline = Some(
                NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .map_err(|e| format!("invalid date format: {e}"))?,
            );
        }
        None => {
            let deadline_str: String = Input::new()
                .with_prompt("Deadline (YYYY-MM-DD, leave empty to skip)")
                .default(String::new())
                .show_default(false)
                .interact_text()
                .map_err(|e| e.to_string())?;
            if !deadline_str.is_empty() {
                entry.deadline = Some(
                    NaiveDate::parse_from_str(&deadline_str, "%Y-%m-%d")
                        .map_err(|e| format!("invalid date format: {e}"))?,
                );
            }
        }
    }

    match tags_flag {
        Some(t) => {
            let parsed: Vec<String> = t.split(',').map(|s| s.to_string()).collect();
            entry.tags = normalize_tags(&parsed);
        }
        None => {
            let tags_str: String = Input::new()
                .with_prompt("Tags (comma-separated, leave empty to skip)")
                .default(String::new())
                .show_default(false)
                .interact_text()
                .map_err(|e| e.to_string())?;
            if !tags_str.is_empty() {
                let parsed: Vec<String> = tags_str.split(',').map(|t| t.to_string()).collect();
                entry.tags = normalize_tags(&parsed);
            }
        }
    }

    entry.normalize()?;
    Ok(entry)
}
