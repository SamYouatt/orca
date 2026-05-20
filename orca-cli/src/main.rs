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
        cli::Commands::Sync {
            workspace,
            verbose,
            force,
        } => commands::sync(&base_dir, workspace.as_deref(), verbose, force)?,
        cli::Commands::Critique => commands::critique(&base_dir)?,
        cli::Commands::Issue { command } => match command {
            cli::IssueCommands::Create { title, body, repo } => {
                let id = commands::issue::create(&base_dir, repo.as_deref(), &title, &body)?;
                println!("{id}");
            }
            cli::IssueCommands::Show { id, repo, json } => {
                let issue = if json {
                    commands::issue::show_json(&base_dir, repo.as_deref(), &id)?
                } else {
                    commands::issue::show(&base_dir, repo.as_deref(), &id)?
                };
                println!("{issue}");
            }
            cli::IssueCommands::List { repo, status, json } => {
                let issues = commands::issue::list(&base_dir, repo.as_deref(), &status, json)?;
                println!("{issues}");
            }
        },
    }

    Ok(())
}
