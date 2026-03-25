use std::path::Path;

use anyhow::Result;
use chrono::Utc;

use crate::{
    git, github, names,
    workspace::{self, WorkspaceConfig},
};

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

pub fn status(base_dir: &Path) -> Result<()> {
    let workspaces = workspace::list_all(base_dir)?;

    if workspaces.is_empty() {
        println!("no workspaces");
        return Ok(());
    }

    let gh_available = github::is_available();
    if !gh_available {
        eprintln!("install gh for PR details");
    }

    println!("Workspaces:\n");

    let name_width = workspaces
        .iter()
        .map(|(n, _)| n.len())
        .max()
        .unwrap()
        .max(4);

    for (name, config) in &workspaces {
        let worktree_path = workspace::worktree_path(base_dir, name);
        let branch = git::worktree_branch(&worktree_path);
        let dirty = git::has_uncommitted_changes(&worktree_path);
        let ab = git::ahead_behind(&worktree_path);
        let has_upstream = ab.is_some();

        let mut parts: Vec<String> = Vec::new();

        match ab {
            Some((0, 0)) => parts.push("clean".to_string()),
            Some((ahead, behind)) => {
                let mut s = String::new();
                if ahead > 0 {
                    s.push_str(&format!("↑{}", ahead));
                }
                if behind > 0 {
                    if !s.is_empty() {
                        s.push(' ');
                    }
                    s.push_str(&format!("↓{}", behind));
                }
                parts.push(s);
            }
            None => {}
        }

        if dirty {
            parts.push("*".to_string());
        }

        let mut check_icon: Option<&str> = None;

        if gh_available {
            match github::pr_for_branch(&config.repo, &branch) {
                Ok(Some(pr)) => {
                    if let Some(checks) = &pr.check_status {
                        check_icon = Some(match checks {
                            github::CheckStatus::Passing => "󱓏",
                            github::CheckStatus::Failing => "󱓌",
                            github::CheckStatus::Pending => "󱓎",
                        });
                    }
                    let pr_str = if pr.state == "MERGED" {
                        format!(" #{}", pr.number)
                    } else {
                        format!("PR #{}", pr.number)
                    };
                    parts.push(pr_str);
                }
                Ok(None) => {}
                Err(e) if !e.contains("no git remotes") => {
                    parts.push(format!("gh: {}", e));
                }
                Err(_) => {}
            }
        }

        let icon = check_icon.unwrap_or(if has_upstream { "\u{e0a0}" } else { " " });
        let status_str = parts.join("  ");

        println!(
            "  {:<name_width$}    {} {}  {}",
            name, icon, branch, status_str,
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
