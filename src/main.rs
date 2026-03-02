mod cli;
mod commands;
mod entry;
mod extensions;
mod git;
mod store;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};
use std::ffi::OsString;
use std::process;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        None => commands::ls::run(None, "namespace", false, None, None, None, &[], None, None, None),
        Some(Commands::Init { remote }) => commands::init::run(remote),
        Some(Commands::Ls {
            ref namespace_prefix,
            ref group,
            json,
            ref kind,
            ref priority,
            ref status,
            ref tag,
            ref deadline,
            ref before,
            ref after,
        }) => commands::ls::run(
            namespace_prefix.as_deref(),
            group,
            json,
            kind.as_deref(),
            priority.as_deref(),
            status.as_deref(),
            tag,
            deadline.as_deref(),
            before.as_deref(),
            after.as_deref(),
        ),
        Some(Commands::Show {
            ref identifier,
            json,
        }) => commands::show::run(identifier, json),
        Some(Commands::Insert {
            ref identifier,
            ref title,
            ref namespace,
            ref kind,
            ref description,
            ref priority,
            ref status,
            ref deadline,
            ref tags,
        }) => commands::insert::run(
            identifier,
            title.as_deref(),
            namespace.as_deref(),
            kind.as_deref(),
            description.as_deref(),
            priority.as_deref(),
            status.as_deref(),
            deadline.as_deref(),
            tags.as_deref(),
        ),
        Some(Commands::Edit {
            ref identifier,
            ref title,
            ref namespace,
            ref kind,
            ref description,
            ref append_description,
            ref priority,
            ref status,
            ref deadline,
            ref tags,
        }) => commands::edit::run(
            identifier,
            title.as_deref(),
            namespace.as_deref(),
            kind.as_deref(),
            description.as_deref(),
            append_description.as_deref(),
            priority.as_deref(),
            status.as_deref(),
            deadline.as_deref(),
            tags.as_deref(),
        ),
        Some(Commands::Rm {
            ref identifier,
            force,
        }) => commands::rm::run(identifier, force),
        Some(Commands::Find { ref pattern, json }) => commands::find::run(pattern, json),
        Some(Commands::Grep { ref pattern, json }) => commands::grep::run(pattern, json),
        Some(Commands::Git { ref args }) => commands::git_cmd::run(args),
        Some(Commands::External(ref args)) => run_external_subcommand(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run_external_subcommand(args: &[OsString]) -> Result<(), String> {
    if args.is_empty() {
        return Ok(());
    }

    let subcommand = args[0].to_string_lossy();
    let binary = format!("mind-{subcommand}");

    #[cfg(unix)]
    {
        use std::io::ErrorKind;
        use std::os::unix::process::CommandExt;
        let err = process::Command::new(&binary).args(&args[1..]).exec();
        if err.kind() == ErrorKind::NotFound {
            emit_unknown_subcommand_and_exit(&subcommand);
        }
        return Err(format!("failed to execute '{binary}': {err}"));
    }

    #[cfg(not(unix))]
    {
        use std::io::ErrorKind;
        match process::Command::new(&binary).args(&args[1..]).status() {
            Ok(status) => {
                process::exit(status.code().unwrap_or(1));
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {
                emit_unknown_subcommand_and_exit(&subcommand);
            }
            Err(err) => {
                return Err(format!("failed to execute '{binary}': {err}"));
            }
        }
    }
}

fn emit_unknown_subcommand_and_exit(subcommand: &str) -> ! {
    let mut cmd = Cli::command();
    cmd.error(
        clap::error::ErrorKind::InvalidSubcommand,
        format!("unrecognized subcommand '{subcommand}'"),
    )
    .exit();
}
