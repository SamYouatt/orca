use anyhow::Result;
use std::path::Path;

use crate::{git, theme, workspace};

pub fn rm(base_dir: &Path, names: &[String]) -> Result<()> {
    for name in names {
        let config = workspace::load(base_dir, name)?;
        let worktree_path = workspace::worktree_path(base_dir, name);
        if worktree_path.exists() {
            git::remove_worktree(&config.repo, &worktree_path)?;
        } else {
            eprintln!("Worktree already removed, cleaning up config");
        }
        workspace::delete(base_dir, name)?;
        println!("Removed workspace {}", theme::blue_bold(name));
    }
    Ok(())
}
