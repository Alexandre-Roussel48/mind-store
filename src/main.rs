mod cli;
mod commands;
mod entry;
mod git;
mod store;

use clap::Parser;
use cli::{Cli, Commands};
use std::process;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        None => commands::ls::run(None, false),
        Some(Commands::Init { remote }) => commands::init::run(remote),
        Some(Commands::Ls { ref subfolder, json }) => commands::ls::run(subfolder.as_deref(), json),
        Some(Commands::Show { ref name, json }) => commands::show::run(name, json),
        Some(Commands::Insert {
            ref name,
            ref kind,
            ref description,
            ref priority,
            ref deadline,
            ref tags,
        }) => commands::insert::run(
            name,
            kind.as_deref(),
            description.as_deref(),
            priority.as_deref(),
            deadline.as_deref(),
            tags.as_deref(),
        ),
        Some(Commands::Edit {
            ref name,
            ref kind,
            ref description,
            ref append_description,
            ref priority,
            ref status,
            ref deadline,
            ref tags,
        }) => commands::edit::run(
            name,
            kind.as_deref(),
            description.as_deref(),
            append_description.as_deref(),
            priority.as_deref(),
            status.as_deref(),
            deadline.as_deref(),
            tags.as_deref(),
        ),
        Some(Commands::Rm {
            ref name,
            recursive,
            force,
        }) => commands::rm::run(name, recursive, force),
        Some(Commands::Find { ref pattern, json }) => commands::find::run(pattern, json),
        Some(Commands::Grep { ref pattern, json }) => commands::grep::run(pattern, json),
        Some(Commands::Git { ref args }) => commands::git_cmd::run(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
