use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

pub fn repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to run git")?;

    if !output.status.success() {
        bail!("not a git repository");
    }

    let path = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(PathBuf::from(path))
}

pub fn create_worktree(repo: &Path, worktree_path: &Path, branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args([
            "-C",
            &repo.display().to_string(),
            "worktree",
            "add",
            "-b",
            branch,
            &worktree_path.display().to_string(),
        ])
        .output()
        .context("failed to run git worktree add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git worktree add failed: {}", stderr.trim());
    }

    Ok(())
}

pub fn remove_worktree(repo: &Path, worktree_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args([
            "-C",
            &repo.display().to_string(),
            "worktree",
            "remove",
            &worktree_path.display().to_string(),
        ])
        .output()
        .context("failed to run git worktree remove")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git worktree remove failed: {}", stderr.trim());
    }

    Ok(())
}
