use std::path::Path;

use anyhow::Result;
use chrono::Utc;

use crate::{
    git, names,
    workspace::{self, WorkspaceConfig},
};

pub fn new(base_dir: &Path) -> Result<()> {
    let repo = git::repo_root()?;
    let candidate = names::generate();
    let name = workspace::resolve_unique_name(base_dir, &candidate);

    let worktree_path = workspace::worktree_path(base_dir, &name);
    git::create_worktree(&repo, &worktree_path, &name)?;

    let config = WorkspaceConfig {
        repo,
        created: Utc::now(),
    };

    workspace::save(base_dir, &name, &config)?;

    println!(
        "created workspace '{}' at {}",
        name,
        worktree_path.display()
    );
    Ok(())
}

pub fn ls(base_dir: &Path) -> Result<()> {
    let workspaces = workspace::list_all(base_dir)?;

    if workspaces.is_empty() {
        println!("no workspaces");
        return Ok(());
    }

    let entries: Vec<_> = workspaces
        .iter()
        .map(|(name, config)| {
            let worktree_path = workspace::worktree_path(base_dir, name);
            let repo_name = git::repo_name(&config.repo);
            let branch = git::worktree_branch(&worktree_path);
            let repo_branch = format!("{}/{}", repo_name, branch);
            (name, repo_branch, config)
        })
        .collect();

    let name_width = entries
        .iter()
        .map(|(n, _, _)| n.len())
        .max()
        .unwrap()
        .max(4);
    let rb_width = entries
        .iter()
        .map(|(_, rb, _)| rb.len())
        .max()
        .unwrap()
        .max(11);

    println!(
        "{:<name_width$}  {:<rb_width$}  CREATED",
        "NAME", "REPO/BRANCH"
    );

    for (name, repo_branch, config) in &entries {
        println!(
            "{:<name_width$}  {:<rb_width$}  {}",
            name,
            repo_branch,
            config.created.format("%Y-%m-%d %H:%M"),
        );
    }

    Ok(())
}

pub fn rm(base_dir: &Path, name: &str) -> Result<()> {
    let config = workspace::load(base_dir, name)?;
    let worktree_path = workspace::worktree_path(base_dir, name);
    if worktree_path.exists() {
        git::remove_worktree(&config.repo, &worktree_path)?;
    } else {
        eprintln!("worktree already removed, cleaning up config");
    }
    workspace::delete(base_dir, name)?;
    println!("removed workspace '{}'", name);
    Ok(())
}
