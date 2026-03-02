# mind-store

A lightweight, Git-backed store for organizing ideas and todos from the command line. Inspired by [pass](https://www.passwordstore.org/) (password-store).

Each entry is stored as a flat TOML file in `~/.mind-store/`, with metadata for title, kind, priority, status, tags, optional deadline, and optional namespace. Every change is auto-committed to Git for full history and sync.

## Install

```bash
# Requires Rust toolchain and a C linker (build-essential)
sudo apt install build-essential   # Debian/Ubuntu
cargo install --path .
```

## Usage

```
mind init [--remote <url>]     Initialize the store (optionally set a Git remote)
mind insert <identifier>       Create a new entry (flat slug or namespace/slug compatibility)
mind show <identifier> [--json] Display an entry
mind edit <identifier>         Edit an existing entry
mind rm [-f] <identifier>      Remove an entry
mind ls [namespace_prefix] [--group <group>] [--json] [--kind <kind>] [--priority <priority>] [--status <status>] [--tag <tag>] [--deadline <date>] [--before <date>] [--after <date>]
mind find <pattern> [--json]   Search entry names
mind grep <pattern> [--json]   Search entry contents
mind git <args...>             Run git commands in the store
```

## External Subcommands

If a subcommand is not built into `mind`, the CLI tries to execute an external
binary named `mind-<subcommand>` from your `PATH`.

Examples:

```bash
mind calendar list
# Executes: mind-calendar list
```

If no matching binary exists, `mind` returns the normal unknown-subcommand error.

`mind edit <identifier>` is interactive by default. If you pass one or more field flags
(`--priority`, `--status`, `--deadline`, etc.), it updates only those fields and does not prompt.
Use `--description` to replace the description, or `--append-description` to append text.

Examples:

```bash
mind insert personal/garage/rework --kind todo --priority high
mind insert rework --namespace personal/garage --kind todo

mind edit personal/garage/rework --status done --deadline 2026-12-20
mind edit rework --namespace none
mind edit rework --append-description "Follow-up note"

mind ls --kind todo --priority high
mind ls --status active --tag rust
mind ls --tag rust --tag cli
mind ls --before 2026-03-15
mind ls personal --after 2026-03-01 --json
mind ls --group namespace
mind ls --group status
mind ls --group tags
```

Running `mind` with no arguments is equivalent to `mind ls`.

## Protocol rules

- Storage is flat: every entry is `<slug>.toml` at the root of the store.
- Slug is an identifier only, not a hierarchy.
- Namespace is the single hierarchy field (`personal/garage`).
- Tags are cross-cutting labels and separate from namespace.
- `mind ls` renders virtual trees from metadata.

## `ls` grouping behavior

`mind ls` defaults to `--group namespace`.

Supported groups:

- `--group namespace` (default)
- `--group kind`
- `--group priority`
- `--group status`
- `--group tags`

Ungrouped values are always shown under `ungrouped/` and rendered last.

Namespace prefix filtering:

```bash
mind ls personal
```

This matches `namespace = "personal"` and children like `personal/garage`.

## Entry format

```toml
title = "Garage rework"
kind = "idea"          # idea | todo | note
description = "..."    # optional
priority = "high"      # low | medium | high | critical
status = "active"      # active | done | archived
namespace = "personal/garage" # optional
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

## Extensions

You can run extensions automatically on write events (`insert`, `edit`, `rm`).

Configuration file:

- `~/.config/mind-store/config.toml`

Example:

```toml
[extensions]
binaries = ["mind-calendar"]
```

Invocation contract:

- `mind insert personal/foo` => `mind-calendar insert foo` (slug is passed)
- `mind edit personal/foo` => `mind-calendar edit foo`
- `mind rm personal/foo` => `mind-calendar rm foo`

Extension failures are best-effort: `mind` prints a warning and continues.

## Configuration

Set `MIND_STORE_DIR` to override the default store location (`~/.mind-store`).

## License

GNU GPL v3