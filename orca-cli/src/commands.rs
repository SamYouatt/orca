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

    let name_width = workspaces
        .iter()
        .map(|(n, _)| n.len())
        .max()
        .unwrap()
        .max(4);
    let repo_width = workspaces
        .iter()
        .map(|(_, c)| c.repo.display().to_string().len())
        .max()
        .unwrap()
        .max(4);

    println!("{:<name_width$}  {:<repo_width$}  CREATED", "NAME", "REPO",);

    for (name, config) in &workspaces {
        println!(
            "{:<name_width$}  {:<repo_width$}  {}",
            name,
            config.repo.display(),
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
