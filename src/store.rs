use std::path::{Component, Path, PathBuf};
use std::{env, fs, io};

const DEFAULT_STORE_DIR: &str = ".mind-store";
const ENV_VAR: &str = "MIND_STORE_DIR";

pub fn store_dir() -> PathBuf {
    if let Ok(dir) = env::var(ENV_VAR) {
        return PathBuf::from(dir);
    }
    let home = env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join(DEFAULT_STORE_DIR)
}

pub fn ensure_store_exists() -> io::Result<()> {
    let dir = store_dir();
    if !dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Mind store not found at {}. Run `mind init` first.",
                dir.display()
            ),
        ));
    }
    Ok(())
}

fn validate_relative_path(input: &str) -> Result<PathBuf, String> {
    if input.trim().is_empty() {
        return Err("path cannot be empty".to_string());
    }

    let path = Path::new(input);
    if path.is_absolute() {
        return Err("absolute paths are not allowed".to_string());
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err("path traversal ('..') is not allowed".to_string());
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err("invalid path component".to_string());
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err("path cannot be empty".to_string());
    }

    Ok(normalized)
}

/// Resolve an entry name (e.g. "project/idea") to a full path with .toml extension.
pub fn entry_path(name: &str) -> Result<PathBuf, String> {
    let rel = validate_relative_path(name)?;
    let mut path = store_dir().join(rel);
    if path.extension().is_none() {
        path.set_extension("toml");
    }
    Ok(path)
}

/// Resolve a subpath inside the store (for commands like ls/rm folder targets).
pub fn store_subpath(path: &str) -> Result<PathBuf, String> {
    let rel = validate_relative_path(path)?;
    Ok(store_dir().join(rel))
}

/// Strip store prefix and .toml extension to get the entry name.
pub fn entry_name(path: &Path) -> String {
    let store = store_dir();
    let rel = path.strip_prefix(&store).unwrap_or(path);
    rel.with_extension("").to_string_lossy().to_string()
}

/// Create parent directories for an entry path if needed.
pub fn ensure_parent_dirs(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
