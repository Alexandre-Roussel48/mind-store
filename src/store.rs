use crate::entry::{normalize_namespace, Entry};
use std::path::{Path, PathBuf};
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

pub fn validate_slug(input: &str) -> Result<String, String> {
    let trimmed = input.trim().trim_end_matches(".toml");
    if trimmed.is_empty() {
        return Err("slug cannot be empty".to_string());
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err("slug cannot contain path separators".to_string());
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(
            "slug can only contain letters, numbers, '-', '_' and '.'".to_string(),
        );
    }
    Ok(trimmed.to_string())
}

pub fn parse_namespace_and_slug(input: &str) -> Result<(Option<String>, String), String> {
    let trimmed = input.trim().trim_matches('/');
    if trimmed.is_empty() {
        return Err("entry identifier cannot be empty".to_string());
    }
    if let Some((namespace, slug)) = trimmed.rsplit_once('/') {
        let namespace = normalize_namespace(namespace)?;
        let slug = validate_slug(slug)?;
        Ok((namespace, slug))
    } else {
        Ok((None, validate_slug(trimmed)?))
    }
}

pub fn entry_path(slug_or_identifier: &str) -> Result<PathBuf, String> {
    let (_, slug) = parse_namespace_and_slug(slug_or_identifier)?;
    Ok(store_dir().join(format!("{slug}.toml")))
}

pub fn entry_slug(path: &Path) -> Option<String> {
    if !path.is_file() || !path.extension().is_some_and(|ext| ext == "toml") {
        return None;
    }
    path.file_stem().map(|s| s.to_string_lossy().to_string())
}

pub fn list_entry_paths() -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let dir = store_dir();
    let rd = fs::read_dir(&dir).map_err(|e| format!("failed to read store '{}': {e}", dir.display()))?;
    for item in rd {
        let item = item.map_err(|e| e.to_string())?;
        let path = item.path();
        let hidden = path
            .file_name()
            .map(|n| n.to_string_lossy().starts_with('.'))
            .unwrap_or(false);
        if hidden {
            continue;
        }
        if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

pub fn load_entry(path: &Path) -> Result<Entry, String> {
    let contents =
        fs::read_to_string(path).map_err(|e| format!("failed to read entry file '{}': {e}", path.display()))?;
    Entry::from_toml(&contents)
        .map_err(|e| format!("failed to parse entry '{}': {e}", path.display()))
}

pub fn resolve_identifier(name: &str) -> Result<(PathBuf, Option<String>, String), String> {
    let (namespace, slug) = parse_namespace_and_slug(name)?;
    Ok((entry_path(&slug)?, namespace, slug))
}

#[cfg(test)]
mod tests {
    use super::{parse_namespace_and_slug, validate_slug};

    #[test]
    fn validate_slug_accepts_flat_identifiers() {
        assert_eq!(validate_slug("rework").unwrap(), "rework");
        assert_eq!(validate_slug("project-x_1").unwrap(), "project-x_1");
    }

    #[test]
    fn validate_slug_rejects_path_separators() {
        assert!(validate_slug("personal/rework").is_err());
        assert!(validate_slug("../rework").is_err());
    }

    #[test]
    fn parse_namespace_and_slug_splits_compat_identifier() {
        let (namespace, slug) = parse_namespace_and_slug("personal/garage/rework").unwrap();
        assert_eq!(namespace.as_deref(), Some("personal/garage"));
        assert_eq!(slug, "rework");
    }

    #[test]
    fn parse_namespace_and_slug_slug_only() {
        let (namespace, slug) = parse_namespace_and_slug("rework").unwrap();
        assert_eq!(namespace, None);
        assert_eq!(slug, "rework");
    }
}
