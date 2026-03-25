use clap::Parser;

use orca::commands;

mod cli;

fn main() -> anyhow::Result<()> {
    let base_dir = dirs::home_dir()
        .expect("could not determine home directory")
        .join(".orca");

    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::New => commands::new(&base_dir)?,
        cli::Commands::Ls => commands::ls(&base_dir)?,
        cli::Commands::Rm { name } => commands::rm(&base_dir, &name)?,
    }

    Ok(())
}
