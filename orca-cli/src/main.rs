use clap::Parser;

use orca::commands;

mod cli;

fn main() -> anyhow::Result<()> {
    let base_dir = orca::base_dir();

    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::New { branch } => commands::new(&base_dir, branch.as_deref())?,
        cli::Commands::Ls => commands::ls(&base_dir)?,
        cli::Commands::Status { porcelain } => commands::status(&base_dir, porcelain)?,
        cli::Commands::Rm { names } => commands::rm(&base_dir, &names)?,
        cli::Commands::Collection => commands::collection(&base_dir)?,
        cli::Commands::Sync { workspace, verbose } => {
            commands::sync(&base_dir, workspace.as_deref(), verbose)?
        }
    }

    Ok(())
}
