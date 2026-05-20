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
            cli::IssueCommands::List {
                repo,
                status,
                blocked_by,
                json,
            } => {
                let issues = commands::issue::list(
                    &base_dir,
                    repo.as_deref(),
                    &status,
                    blocked_by.as_deref(),
                    json,
                )?;
                println!("{issues}");
            }
            cli::IssueCommands::Block { id, blockers, repo } => {
                let blocker_refs = blockers.iter().map(String::as_str).collect::<Vec<_>>();
                commands::issue::block(&base_dir, repo.as_deref(), &id, &blocker_refs)?;
            }
            cli::IssueCommands::Unblock { id, blockers, repo } => {
                let blocker_refs = blockers.iter().map(String::as_str).collect::<Vec<_>>();
                commands::issue::unblock(&base_dir, repo.as_deref(), &id, &blocker_refs)?;
            }
            cli::IssueCommands::Update {
                id,
                title,
                status,
                body,
                blockers,
                add_blockers,
                remove_blockers,
                repo,
            } => {
                let blocker_update = match (
                    blockers,
                    add_blockers.is_empty(),
                    remove_blockers.is_empty(),
                ) {
                    (Some(blockers), true, true) => {
                        commands::issue::BlockerUpdate::Replace(blockers)
                    }
                    (None, false, true) => commands::issue::BlockerUpdate::Add(add_blockers),
                    (None, true, false) => commands::issue::BlockerUpdate::Remove(remove_blockers),
                    (None, true, true) => commands::issue::BlockerUpdate::Unchanged,
                    _ => unreachable!("clap rejects mixed blocker update modes"),
                };
                commands::issue::update(
                    &base_dir,
                    repo.as_deref(),
                    &id,
                    commands::issue::IssueUpdate {
                        title,
                        status,
                        body,
                        blockers: blocker_update,
                    },
                )?;
            }
        },
    }

    Ok(())
}
