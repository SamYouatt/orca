use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new workspace based on this repo
    New,
    /// Lists all workspaces
    Ls,
    /// Remove a workspace
    Rm {
        /// Name of the workspace to remove
        name: String,
    },
}
