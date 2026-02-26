use colored::Colorize;
use serde::Serialize;
use std::fs;
use walkdir::WalkDir;

use crate::store;

#[derive(Serialize)]
struct GrepMatch {
    name: String,
    lines: Vec<String>,
}

pub fn run(pattern: &str, json: bool) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let store_root = store::store_dir();
    let pattern_lower = pattern.to_lowercase();
    let mut matches: Vec<GrepMatch> = Vec::new();

    for result in WalkDir::new(&store_root)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
    {
        let entry = result.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "toml") {
            let contents =
                fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;

            if contents.to_lowercase().contains(&pattern_lower) {
                let name = store::entry_name(path);
                let mut matching_lines = Vec::new();
                for line in contents.lines() {
                    if line.to_lowercase().contains(&pattern_lower) {
                        matching_lines.push(line.to_string());
                    }
                }
                if !matching_lines.is_empty() {
                    matches.push(GrepMatch {
                        name,
                        lines: matching_lines,
                    });
                }
            }
        }
    }

    if json {
        let output = serde_json::to_string_pretty(&matches)
            .map_err(|e| format!("failed to serialize JSON: {e}"))?;
        println!("{output}");
    } else if matches.is_empty() {
        println!("No entries contain '{pattern}'.");
    } else {
        for m in matches {
            println!("{}: ", m.name.bold());
            for line in m.lines {
                println!("  {line}");
            }
        }
    }

    Ok(())
}
