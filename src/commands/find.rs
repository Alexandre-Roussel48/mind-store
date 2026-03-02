use crate::store;

pub fn run(pattern: &str, json: bool) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let pattern_lower = pattern.to_lowercase();
    let mut matches = Vec::new();

    for path in store::list_entry_paths()? {
        let Some(slug) = store::entry_slug(&path) else {
            continue;
        };
        let entry = store::load_entry(&path)?;
        let namespace = entry.namespace.unwrap_or_default();
        let identifier = if namespace.is_empty() {
            slug.clone()
        } else {
            format!("{namespace}/{slug}")
        };
        let searchable = format!("{slug} {} {namespace}", entry.title).to_lowercase();
        if searchable.contains(&pattern_lower) {
            matches.push(identifier);
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
