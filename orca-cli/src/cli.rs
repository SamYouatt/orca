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
        #[arg(long)]
        no_script: bool,
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
        #[arg(long)]
        no_script: bool,
    },
    /// View your fish collection
    Collection,
    /// Bidirectionally sync files between a workspace and its root repo
    Sync {
        /// Name of the workspace to sync (detected from cwd if omitted)
        #[arg(short, long)]
        workspace: Option<String>,
        /// Show individual file sync events
        #[arg(short, long)]
        verbose: bool,
        /// Allow sync even if root has uncommitted changes
        #[arg(short, long)]
        force: bool,
    },
    /// Open interactive code review in the browser
    Critique,
    /// Create and view repository-scoped issues
    Issue {
        #[command(subcommand)]
        command: IssueCommands,
    },
}

#[derive(Subcommand)]
pub enum IssueCommands {
    /// Create an issue in the resolved repository
    Create {
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "")]
        body: String,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    /// Show an issue from the resolved repository
    Show {
        id: String,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// List issues from the resolved repository
    List {
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        #[arg(long)]
        status: Vec<String>,
        #[arg(long)]
        blocked_by: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Add blockers to an issue
    Block {
        id: String,
        blockers: Vec<String>,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    /// Remove blockers from an issue
    Unblock {
        id: String,
        blockers: Vec<String>,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    /// Patch issue fields and blockers
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long, num_args = 0.., value_delimiter = ',', conflicts_with_all = ["add_blockers", "remove_blockers"])]
        blockers: Option<Vec<String>>,
        #[arg(long, num_args = 1.., value_delimiter = ',', conflicts_with_all = ["blockers", "remove_blockers"])]
        add_blockers: Vec<String>,
        #[arg(long, num_args = 1.., value_delimiter = ',', conflicts_with_all = ["blockers", "add_blockers"])]
        remove_blockers: Vec<String>,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
}
