use anyhow::Result;
use chrono::Utc;
use colored::Colorize;
use std::path::Path;

use crate::collection;
use crate::git;
use crate::names::{self, Rarity};
use crate::setup::{self, ScriptContext};
use crate::theme;
use crate::workspace;
use crate::workspace::WorkspaceConfig;

pub fn new(base_dir: &Path, branch: Option<&str>) -> Result<()> {
    let repo = git::repo_root()?;
    let catch = names::generate();
    let name = workspace::resolve_unique_name(base_dir, &catch.name);
    let branch = branch.unwrap_or(&name);

    let worktree_path = workspace::worktree_path(base_dir, &name);
    git::create_worktree(&repo, &worktree_path, branch)?;

    let config = WorkspaceConfig {
        repo: repo.clone(),
        created: Utc::now(),
    };

    workspace::save(base_dir, &name, &config)?;

    setup::run_setup_scripts(
        base_dir,
        &repo,
        &ScriptContext {
            workspace_name: &name,
            branch_name: branch,
            workspace_path: &worktree_path,
        },
    );

    println!(
        "Created workspace {} on branch {} at {}",
        theme::blue_bold(&name),
        theme::purple(branch),
        theme::blue(&worktree_path.display().to_string())
    );

    let repo_name = match workspace::detect_current(base_dir) {
        Some(ws) => workspace::load(base_dir, &ws)
            .map(|c| git::repo_name(&c.repo))
            .unwrap_or_else(|_| git::repo_name(&repo)),
        None => git::repo_name(&repo),
    };
    let is_new = collection::record_catch(base_dir, &catch.name, catch.rarity, &repo_name);
    let new_tag = if is_new {
        format!(" {}", "(NEW)".bold())
    } else {
        String::new()
    };

    match catch.rarity {
        Rarity::Common => {
            println!("  caught {}{}", theme::grey(&catch.name), new_tag);
        }
        Rarity::Rare => {
            println!(
                "  {} rare catch: {}{}",
                theme::blue("*"),
                theme::blue(&catch.name),
                new_tag,
            );
        }
        Rarity::Epic => {
            println!(
                "  {} epic catch: {}!{}",
                theme::purple("**"),
                theme::purple(&catch.name),
                new_tag,
            );
        }
        Rarity::Legendary => {
            println!(
                "  {} LEGENDARY CATCH: {}!! {}{}",
                theme::gold_bold(">>>"),
                theme::gold_bold(&catch.name),
                theme::gold_bold("<<<"),
                new_tag,
            );
        }
    }

    Ok(())
}
