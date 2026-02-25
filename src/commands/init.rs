use std::fs;

use crate::{git, store};

pub fn run(remote: Option<String>) -> Result<(), String> {
    let dir = store::store_dir();

    if dir.exists() {
        println!("Mind store already exists at {}", dir.display());
    } else {
        fs::create_dir_all(&dir).map_err(|e| format!("failed to create store: {e}"))?;
        println!("Created mind store at {}", dir.display());
    }

    git::init()?;
    println!("Initialized git repository.");

    if let Some(url) = remote {
        git::set_remote(&url)?;
        println!("Remote set to {url}");
    }

    Ok(())
}
