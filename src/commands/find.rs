use walkdir::WalkDir;

use crate::store;

pub fn run(pattern: &str, json: bool) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let store_root = store::store_dir();
    let pattern_lower = pattern.to_lowercase();
    let mut matches = Vec::new();

    for result in WalkDir::new(&store_root)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
    {
        let entry = result.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "toml") {
            let name = store::entry_name(path);
            if name.to_lowercase().contains(&pattern_lower) {
                matches.push(name);
            }
        }
    }

    if json {
        let output = serde_json::to_string_pretty(&matches)
            .map_err(|e| format!("failed to serialize JSON: {e}"))?;
        println!("{output}");
    } else if matches.is_empty() {
        println!("No entries matching '{pattern}'.");
    } else {
        for name in matches {
            println!("{name}");
        }
    }

    Ok(())
}
