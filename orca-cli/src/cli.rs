use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new workspace based on this repo
    New {
        #[arg(short, long)]
        branch: Option<String>,
    },
    /// Lists all workspaces
    Ls,
    /// Show workspace status with git and GitHub info
    Status {
        #[arg(long)]
        porcelain: bool,
    },
    /// Remove a workspace
    Rm {
        /// Names of the workspaces to remove
        names: Vec<String>,
    },
}
