# mind-store

A lightweight, Git-backed store for organizing ideas and todos from the command line. Inspired by [pass](https://www.passwordstore.org/) (password-store).

Each entry is stored as a TOML file in `~/.mind-store/`, with metadata for kind, priority, status, tags, and optional deadline. Every change is auto-committed to Git for full history and sync.

## Install

```bash
# Requires Rust toolchain and a C linker (build-essential)
sudo apt install build-essential   # Debian/Ubuntu
cargo install --path .
```

## Usage

```
mind init [--remote <url>]     Initialize the store (optionally set a Git remote)
mind insert <name>             Create a new entry (interactive prompts)
mind show <name>               Display an entry
mind edit <name>               Edit an existing entry
mind rm [-r] [-f] <name>       Remove an entry or folder recursively
mind ls [subfolder/] [--json]  List entries (tree view or JSON)
mind find <pattern>            Search entry names
mind grep <pattern>            Search entry contents
mind git <args...>             Run git commands in the store
```

`mind edit <name>` is interactive by default. If you pass one or more field flags
(`--priority`, `--status`, `--deadline`, etc.), it updates only those fields and does not prompt.
Use `--description` to replace the description, or `--append-description` to append text.

Examples:

```bash
mind edit project-x/idea-1 --priority high
mind edit project-x/idea-1 --status done --deadline 20-12-2026
mind edit project-x/idea-1 --tags none
mind edit project-x/idea-1 --append-description "Follow-up note"
```

Running `mind` with no arguments is equivalent to `mind ls`.

Entry names can include `/` for organization (e.g. `mind insert project-x/feature-idea`).

## Entry format

```toml
name = "my-idea"
kind = "idea"          # idea | todo | note
description = "..."
priority = "medium"    # low | medium | high | critical
status = "active"      # active | done | archived
created = "2026-02-23T10:00:00Z"
updated = "2026-02-23T10:00:00Z"
tags = ["rust", "cli"]
deadline = "2026-03-15"
```

## Sync

```bash
mind git remote add origin git@github.com:user/mind-store.git
mind git push -u origin main
mind git pull
```

## Configuration

Set `MIND_STORE_DIR` to override the default store location (`~/.mind-store`).

## License

GNU GPL v3