use std::process::Command;

use crate::store;

fn run_git(args: &[&str]) -> Result<String, String> {
    let store = store::store_dir();
    let output = Command::new("git")
        .args(args)
        .current_dir(&store)
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git {}: {stderr}", args.join(" ")))
    }
}

pub fn init() -> Result<(), String> {
    run_git(&["init"])?;
    Ok(())
}

pub fn add_and_commit(message: &str) -> Result<(), String> {
    run_git(&["add", "-A"])?;

    // Check if there's anything to commit
    let status = run_git(&["status", "--porcelain"])?;
    if status.trim().is_empty() {
        return Ok(());
    }

    run_git(&["commit", "-m", message])?;
    Ok(())
}

pub fn set_remote(url: &str) -> Result<(), String> {
    // Try adding; if it already exists, set the URL
    if run_git(&["remote", "add", "origin", url]).is_err() {
        run_git(&["remote", "set-url", "origin", url])?;
    }
    Ok(())
}

pub fn passthrough(args: &[String]) -> Result<(), String> {
    let store = store::store_dir();
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let status = Command::new("git")
        .args(&str_args)
        .current_dir(&store)
        .status()
        .map_err(|e| format!("failed to run git: {e}"))?;

    if !status.success() {
        return Err(format!("git {} failed", str_args.join(" ")));
    }
    Ok(())
}
