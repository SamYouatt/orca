use std::collections::HashSet;
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

pub fn fetch_origin(repo: &Path) -> bool {
    Command::new("git")
        .args(["-C", &repo.display().to_string(), "fetch", "origin"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns the remote default branch ref (e.g. "origin/main") if available.
pub fn remote_default_branch(repo: &Path) -> Option<String> {
    let output = Command::new("git")
        .args([
            "-C",
            &repo.display().to_string(),
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "--short",
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() {
            return Some(branch);
        }
    }

    // fallback: try origin/main then origin/master
    for candidate in &["origin/main", "origin/master"] {
        let check = Command::new("git")
            .args([
                "-C",
                &repo.display().to_string(),
                "rev-parse",
                "--verify",
                candidate,
            ])
            .output()
            .ok();
        if check.map(|o| o.status.success()).unwrap_or(false) {
            return Some(candidate.to_string());
        }
    }

    None
}

pub fn create_worktree(
    repo: &Path,
    worktree_path: &Path,
    branch: &str,
    start_point: Option<&str>,
) -> Result<()> {
    let repo_str = repo.display().to_string();
    let wt_str = worktree_path.display().to_string();

    let mut cmd = Command::new("git");
    cmd.args(["-C", &repo_str, "worktree", "add", "-b", branch, &wt_str]);
    if let Some(sp) = start_point {
        cmd.arg(sp);
    }

    let output = cmd.output().context("failed to run git worktree add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git worktree add failed: {}", stderr.trim());
    }

    Ok(())
}

pub fn repo_name(repo: &Path) -> String {
    repo.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

pub fn worktree_branch(worktree_path: &Path) -> String {
    Command::new("git")
        .args([
            "-C",
            &worktree_path.display().to_string(),
            "rev-parse",
            "--abbrev-ref",
            "HEAD",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn ahead_behind(worktree_path: &Path) -> Option<(u32, u32)> {
    let output = Command::new("git")
        .args([
            "-C",
            &worktree_path.display().to_string(),
            "rev-list",
            "--left-right",
            "--count",
            "HEAD...@{upstream}",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())?;

    let text = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = text.trim().split('\t').collect();
    if parts.len() == 2 {
        Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
    } else {
        None
    }
}

pub fn diff_stat(worktree_path: &Path) -> Option<(u32, u32)> {
    let merge_base = Command::new("git")
        .args([
            "-C",
            &worktree_path.display().to_string(),
            "merge-base",
            "HEAD",
            "HEAD@{upstream}",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())?;

    let base = String::from_utf8_lossy(&merge_base.stdout)
        .trim()
        .to_string();

    let output = Command::new("git")
        .args([
            "-C",
            &worktree_path.display().to_string(),
            "diff",
            "--numstat",
            &base,
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())?;

    let text = String::from_utf8_lossy(&output.stdout);
    let (mut added, mut removed) = (0u32, 0u32);
    for line in text.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            added += parts[0].parse::<u32>().unwrap_or(0);
            removed += parts[1].parse::<u32>().unwrap_or(0);
        }
    }

    Some((added, removed))
}

pub fn has_uncommitted_changes(worktree_path: &Path) -> bool {
    Command::new("git")
        .args([
            "-C",
            &worktree_path.display().to_string(),
            "status",
            "--porcelain",
        ])
        .output()
        .ok()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

pub fn tracked_files(repo: &Path, rel_paths: &[String]) -> HashSet<String> {
    if rel_paths.is_empty() {
        return HashSet::new();
    }
    let mut cmd = Command::new("git");
    cmd.args(["-C", &repo.display().to_string(), "ls-files", "--"]);
    for p in rel_paths {
        cmd.arg(p);
    }
    cmd.output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.to_string())
                .collect()
        })
        .unwrap_or_default()
}

pub fn checkout_files(repo: &Path, rel_paths: &[&str]) -> Result<()> {
    if rel_paths.is_empty() {
        return Ok(());
    }
    let mut cmd = Command::new("git");
    cmd.args(["-C", &repo.display().to_string(), "checkout", "--"]);
    for p in rel_paths {
        cmd.arg(p);
    }
    let output = cmd.output().context("failed to run git checkout")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git checkout failed: {}", stderr.trim());
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
