use dialoguer::Confirm;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{git, store};

pub fn run(name: &str, recursive: bool, force: bool) -> Result<(), String> {
    store::ensure_store_exists().map_err(|e| e.to_string())?;

    let store_root = store::store_dir();
    let dir_path = store::store_subpath(name)?;
    let file_path = store::entry_path(name)?;

    let target = if file_path.exists() {
        Target::File(file_path)
    } else if dir_path.exists() && dir_path.is_dir() {
        if !recursive {
            return Err(format!(
                "'{name}' is a folder. Use `mind rm -r {name}` to remove it recursively."
            ));
        }
        Target::Dir(dir_path)
    } else {
        return Err(format!("Entry or folder '{name}' not found."));
    };

    if let Target::Dir(_) = target {
        if !recursive {
            return Err("Recursive flag is required to remove folders.".to_string());
        }
    }

    if !force {
        let prompt = match &target {
            Target::File(_) => format!("Remove entry '{name}'?"),
            Target::Dir(_) => format!("Remove folder '{name}' recursively?"),
        };
        let confirm = Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .interact()
            .map_err(|e| e.to_string())?;
        if !confirm {
            println!("Aborted.");
            return Ok(());
        }
    }

    match target {
        Target::File(path) => {
            fs::remove_file(&path).map_err(|e| format!("failed to remove entry: {e}"))?;
            cleanup_empty_parents(path.parent(), &store_root);
        }
        Target::Dir(path) => {
            fs::remove_dir_all(&path).map_err(|e| format!("failed to remove folder: {e}"))?;
            cleanup_empty_parents(path.parent(), &store_root);
        }
    }

    git::add_and_commit(&format!("Remove {name}"))?;
    println!("Removed '{name}'.");
    Ok(())
}

enum Target {
    File(PathBuf),
    Dir(PathBuf),
}

fn cleanup_empty_parents(start: Option<&Path>, store_root: &Path) {
    let mut dir = start.map(|p| p.to_path_buf());
    while let Some(d) = dir {
        if d == store_root {
            break;
        }
        if d.read_dir().map(|mut rd| rd.next().is_none()).unwrap_or(false) {
            let _ = fs::remove_dir(&d);
            dir = d.parent().map(|p| p.to_path_buf());
        } else {
            break;
        }
    }
}
