use anyhow::Result;
use std::path::Path;

use crate::git;
use crate::theme;
use crate::workspace;

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

    let header = format!(
        " {:<name_width$}  {:<rb_width$}  {:<16} ",
        "Name", "Repo/Branch", "Created"
    );
    println!("{}", theme::header(&header));

    for (name, repo_branch, config) in &entries {
        println!(
            " {:<name_width$}  {:<rb_width$}  {}",
            name,
            repo_branch,
            theme::grey(&format!("{}", config.created.format("%Y-%m-%d %H:%M"))),
        );
    }

    Ok(())
}
