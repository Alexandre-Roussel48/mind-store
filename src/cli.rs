use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "mind",
    about = "A Git-backed store for ideas and todos",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new mind store
    Init {
        /// Git remote URL to sync with
        #[arg(long)]
        remote: Option<String>,
    },

    /// List entries in the store
    Ls {
        /// Subfolder to list
        subfolder: Option<String>,
        /// Output arborescence as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show an entry
    Show {
        /// Entry name (e.g. "project/my-idea")
        name: String,
    },

    /// Insert a new entry
    Insert {
        /// Entry name (e.g. "project/my-idea")
        name: String,
        /// Kind: idea, todo, note
        #[arg(short, long)]
        kind: Option<String>,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
        /// Priority: low, medium, high, critical
        #[arg(short, long)]
        priority: Option<String>,
        /// [optional] Deadline in DD-MM-YYYY format
        #[arg(long)]
        deadline: Option<String>,
        /// [optional] Comma-separated tags
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// Edit an existing entry (pass any field flags to update only those fields non-interactively)
    Edit {
        /// Entry name
        name: String,
        /// Kind: idea, todo, note
        #[arg(short, long)]
        kind: Option<String>,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
        /// Append text to existing description
        #[arg(long)]
        append_description: Option<String>,
        /// Priority: low, medium, high, critical
        #[arg(short, long)]
        priority: Option<String>,
        /// Status: active, done, archived
        #[arg(short, long)]
        status: Option<String>,
        /// Deadline in DD-MM-YYYY format (use "none" to clear)
        #[arg(long)]
        deadline: Option<String>,
        /// Comma-separated tags (use "none" to clear)
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// Remove an entry
    Rm {
        /// Entry name or folder path
        name: String,
        /// Recursively remove a folder and all entries inside it
        #[arg(short, long)]
        recursive: bool,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Search entry names
    Find {
        /// Pattern to search for
        pattern: String,
    },

    /// Search entry contents
    Grep {
        /// Pattern to search for
        pattern: String,
    },

    /// Run git commands in the store directory
    Git {
        /// Git arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

}
