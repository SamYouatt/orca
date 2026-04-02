use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use crate::git;
use crate::names;
use crate::theme;
use crate::workspace;
use crate::workspace::WorkspaceConfig;

pub fn new(base_dir: &Path, branch: Option<&str>) -> Result<()> {
    let repo = git::repo_root()?;
    let candidate = names::generate();
    let name = workspace::resolve_unique_name(base_dir, &candidate);
    let branch = branch.unwrap_or(&name);

    let worktree_path = workspace::worktree_path(base_dir, &name);
    git::create_worktree(&repo, &worktree_path, branch)?;

    let config = WorkspaceConfig {
        repo,
        created: Utc::now(),
    };

    workspace::save(base_dir, &name, &config)?;

    println!(
        "Created workspace {} on branch {} at {}",
        theme::blue_bold(&name),
        theme::purple(branch),
        theme::blue(&worktree_path.display().to_string())
    );
    Ok(())
}
