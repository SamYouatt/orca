use clap::Parser;

use orca::commands;

mod cli;

fn main() -> anyhow::Result<()> {
    let base_dir = orca::base_dir();

    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::New { branch, no_script } => {
            commands::new(&base_dir, branch.as_deref(), no_script)?
        }
        cli::Commands::Ls => commands::ls(&base_dir)?,
        cli::Commands::Status { porcelain } => commands::status(&base_dir, porcelain)?,
        cli::Commands::Rm { names, no_script } => commands::rm(&base_dir, &names, no_script)?,
        cli::Commands::Collection => commands::collection(&base_dir)?,
        cli::Commands::Sync { workspace, verbose } => {
            commands::sync(&base_dir, workspace.as_deref(), verbose)?
        }
        cli::Commands::Critique => commands::critique(&base_dir)?,
    }

    Ok(())
}
