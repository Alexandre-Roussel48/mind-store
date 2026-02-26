use chrono::NaiveDate;
use dialoguer::{Editor, Input, Select};
use std::fs;

use crate::entry::{Entry, Kind, Priority};
use crate::{git, store};

pub fn run(
    name: &str,
    kind: Option<&str>,
    description: Option<&str>,
    priority: Option<&str>,
    deadline: Option<&str>,
    tags: Option<&str>,
) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let path = store::entry_path(name)?;
    if path.exists() {
        return Err(format!(
            "Entry '{name}' already exists. Use `mind edit {name}` instead."
        ));
    }

    let non_interactive = kind.is_some() && description.is_some() && priority.is_some();

    let entry = if non_interactive {
        build_from_flags(name, kind.unwrap(), description.unwrap(), priority.unwrap(), deadline, tags)?
    } else {
        build_interactive(name, kind, description, priority, deadline, tags)?
    };

    let toml = entry
        .to_toml()
        .map_err(|e| format!("serialization error: {e}"))?;

    store::ensure_parent_dirs(&path).map_err(|e| e.to_string())?;
    fs::write(&path, &toml).map_err(|e| format!("failed to write entry: {e}"))?;

    git::add_and_commit(&format!("mind: add {name}"))?;
    println!("Inserted entry '{name}'.");
    Ok(())
}

fn build_from_flags(
    name: &str,
    kind: &str,
    description: &str,
    priority: &str,
    deadline: Option<&str>,
    tags: Option<&str>,
) -> Result<Entry, String> {
    let kind: Kind = kind.parse()?;
    let priority: Priority = priority.parse()?;

    let mut entry = Entry::new(
        name.to_string(),
        kind,
        description.to_string(),
        priority,
    );

    if let Some(d) = deadline {
        entry.deadline = Some(
            NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|e| format!("invalid date format: {e}"))?,
        );
    }

    if let Some(t) = tags {
        entry.tags = t.split(',').map(|s| s.trim().to_string()).collect();
    }

    Ok(entry)
}

fn build_interactive(
    name: &str,
    kind_flag: Option<&str>,
    desc_flag: Option<&str>,
    priority_flag: Option<&str>,
    deadline_flag: Option<&str>,
    tags_flag: Option<&str>,
) -> Result<Entry, String> {
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
        Some(d) => d.to_string(),
        None => {
            let text = Editor::new()
                .require_save(true)
                .edit("Enter description (save and close editor when done)")
                .map_err(|e| e.to_string())?;
            match text {
                Some(d) if !d.trim().is_empty() => d.trim().to_string(),
                _ => return Err("Description cannot be empty.".to_string()),
            }
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

    let mut entry = Entry::new(name.to_string(), kind, description, priority);

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
            entry.tags = t.split(',').map(|s| s.trim().to_string()).collect();
        }
        None => {
            let tags_str: String = Input::new()
                .with_prompt("Tags (comma-separated, leave empty to skip)")
                .default(String::new())
                .show_default(false)
                .interact_text()
                .map_err(|e| e.to_string())?;
            if !tags_str.is_empty() {
                entry.tags = tags_str.split(',').map(|t| t.trim().to_string()).collect();
            }
        }
    }

    Ok(entry)
}
