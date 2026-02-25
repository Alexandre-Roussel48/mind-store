use colored::Colorize;
use std::fs;
use walkdir::WalkDir;

use crate::store;

pub fn run(pattern: &str) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let store_root = store::store_dir();
    let pattern_lower = pattern.to_lowercase();
    let mut found = false;

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
                println!("{}: ", name.bold());
                for line in contents.lines() {
                    if line.to_lowercase().contains(&pattern_lower) {
                        println!("  {line}");
                    }
                }
                found = true;
            }
        }
    }

    if !found {
        println!("No entries contain '{pattern}'.");
    }

    Ok(())
}
