use colored::Colorize;
use std::fs;

use crate::entry::{format_deadline_ddmmyyyy, Entry};
use crate::store;

pub fn run(name: &str) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let path = store::entry_path(name)?;
    if !path.exists() {
        return Err(format!("Entry '{name}' not found."));
    }

    let contents = fs::read_to_string(&path).map_err(|e| format!("failed to read entry: {e}"))?;
    let entry = Entry::from_toml(&contents).map_err(|e| format!("failed to parse entry: {e}"))?;

    println!("{}", entry.name.bold());
    println!("{}: {}", "Kind".dimmed(), entry.kind);
    println!("{}: {}", "Priority".dimmed(), format_priority(&entry));
    println!("{}: {}", "Status".dimmed(), entry.status);
    println!("{}: {}", "Created".dimmed(), entry.created.format("%Y-%m-%d %H:%M"));
    println!("{}: {}", "Updated".dimmed(), entry.updated.format("%Y-%m-%d %H:%M"));

    if let Some(deadline) = &entry.deadline {
        println!("{}: {}", "Deadline".dimmed(), format_deadline_ddmmyyyy(*deadline));
    }
    if !entry.tags.is_empty() {
        println!("{}: {}", "Tags".dimmed(), entry.tags.join(", "));
    }

    println!();
    println!("{}", entry.description);

    Ok(())
}

fn format_priority(entry: &Entry) -> String {
    use crate::entry::Priority;
    let s = entry.priority.to_string();
    match entry.priority {
        Priority::Low => s.green().to_string(),
        Priority::Medium => s.yellow().to_string(),
        Priority::High => s.red().to_string(),
        Priority::Critical => s.red().bold().to_string(),
    }
}
