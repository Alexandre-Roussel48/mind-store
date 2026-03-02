use dialoguer::Confirm;
use std::fs;

use crate::{extensions, git, store};

pub fn run(identifier: &str, force: bool) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let (path, expected_namespace, slug) = store::resolve_identifier(identifier)?;
    if !path.exists() {
        return Err(format!("Entry '{slug}' not found."));
    }
    if let Some(ns) = expected_namespace {
        let entry = store::load_entry(&path)?;
        if entry.namespace.as_deref() != Some(ns.as_str()) {
            return Err(format!(
                "Entry '{slug}' exists but is in namespace '{}' (requested '{}').",
                entry.namespace.unwrap_or_else(|| "ungrouped".to_string()),
                ns
            ));
        }
    }

    if !force {
        let confirm = Confirm::new()
            .with_prompt(format!("Remove entry '{slug}'?"))
            .default(false)
            .interact()
            .map_err(|e| e.to_string())?;
        if !confirm {
            println!("Aborted.");
            return Ok(());
        }
    }

    fs::remove_file(&path).map_err(|e| format!("failed to remove entry: {e}"))?;

    git::add_and_commit(&format!("mind: remove {slug}"))?;
    extensions::run_event("rm", &slug, None);
    println!("Removed '{slug}'.");
    Ok(())
}
