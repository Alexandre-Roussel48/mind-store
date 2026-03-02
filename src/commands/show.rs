use colored::Colorize;
use std::fs;

use crate::entry::Entry;
use crate::store;

pub fn run(identifier: &str, json: bool) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let (path, expected_namespace, slug) = store::resolve_identifier(identifier)?;
    if !path.exists() {
        return Err(format!("Entry '{slug}' not found."));
    }

    let contents = fs::read_to_string(&path).map_err(|e| format!("failed to read entry: {e}"))?;
    let entry = Entry::from_toml(&contents).map_err(|e| format!("failed to parse entry: {e}"))?;
    if let Some(ns) = expected_namespace {
        if entry.namespace.as_deref() != Some(ns.as_str()) {
            return Err(format!(
                "Entry '{slug}' exists but is in namespace '{}' (requested '{}').",
                entry.namespace.unwrap_or_else(|| "ungrouped".to_string()),
                ns
            ));
        }
    }

    if json {
        let output = serde_json::to_string_pretty(&serde_json::json!({
            "slug": slug,
            "entry": entry
        }))
            .map_err(|e| format!("failed to serialize JSON: {e}"))?;
        println!("{output}");
        return Ok(());
    }

    println!("{}", entry.title.bold());
    println!("{}: {}", "Slug".dimmed(), slug);
    if let Some(namespace) = &entry.namespace {
        println!("{}: {}", "Namespace".dimmed(), namespace);
    } else {
        println!("{}: {}", "Namespace".dimmed(), "ungrouped");
    }
    println!("{}: {}", "Kind".dimmed(), entry.kind);
    println!("{}: {}", "Priority".dimmed(), format_priority(&entry));
    println!("{}: {}", "Status".dimmed(), entry.status);
    println!("{}: {}", "Created".dimmed(), entry.created.format("%Y-%m-%d %H:%M"));
    println!("{}: {}", "Updated".dimmed(), entry.updated.format("%Y-%m-%d %H:%M"));

    if let Some(deadline) = &entry.deadline {
        println!("{}: {}", "Deadline".dimmed(), deadline.format("%Y-%m-%d"));
    }
    if !entry.tags.is_empty() {
        println!("{}: {}", "Tags".dimmed(), entry.tags.join(", "));
    }

    println!();
    if let Some(description) = &entry.description {
        println!("{description}");
    }

    Ok(())
}

fn format_priority(entry: &Entry) -> String {
    use crate::entry::Priority;
    match entry.priority {
        Priority::Low => "low".green().to_string(),
        Priority::Medium => "medium".yellow().to_string(),
        Priority::High => "high".red().to_string(),
        Priority::Critical => "critical".red().bold().to_string(),
    }
}
