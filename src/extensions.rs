use serde::Deserialize;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::entry::Entry;

#[derive(Debug, Deserialize)]
struct ExtensionsConfig {
    extensions: Option<ExtensionList>,
}

#[derive(Debug, Deserialize)]
struct ExtensionList {
    binaries: Option<Vec<String>>,
}

pub fn run_event(event: &str, slug: &str, entry: Option<&Entry>) {
    let binaries = load_extension_binaries();
    let payload = json!({
        "event": event,
        "slug": slug,
        "entry": entry
    })
    .to_string();

    for binary in binaries {
        match Command::new(&binary)
            .arg(event)
            .arg(slug)
            .env("MIND_EVENT_JSON", &payload)
            .status()
        {
            Ok(status) if status.success() => {}
            Ok(status) => {
                eprintln!(
                    "Warning: extension '{}' failed for '{}' on '{}' with exit status {}",
                    binary, event, slug, status
                );
            }
            Err(err) => {
                eprintln!(
                    "Warning: failed to run extension '{}' for '{}' on '{}': {}",
                    binary, event, slug, err
                );
            }
        }
    }
}

fn load_extension_binaries() -> Vec<String> {
    let path = config_path();
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                eprintln!(
                    "Warning: failed to read extension config at '{}': {}",
                    path.display(),
                    err
                );
            }
            return Vec::new();
        }
    };

    match toml::from_str::<ExtensionsConfig>(&contents) {
        Ok(cfg) => cfg
            .extensions
            .and_then(|e| e.binaries)
            .unwrap_or_default()
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .collect(),
        Err(err) => {
            eprintln!(
                "Warning: failed to parse extension config at '{}': {}",
                path.display(),
                err
            );
            Vec::new()
        }
    }
}

fn config_path() -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("mind-store")
            .join("config.toml");
    }
    PathBuf::from(".config/mind-store/config.toml")
}
