use anyhow::Result;
use std::path::Path;

use crate::workspace::{self, WorkspaceConfig};
use crate::{git, github, theme};

struct WorkspaceStatus {
    name: String,
    repo: String,
    icon: String,
    branch: String,
    status: String,
}

fn gather_workspace_status(
    base_dir: &Path,
    name: &str,
    config: &WorkspaceConfig,
    gh_available: bool,
) -> WorkspaceStatus {
    let worktree_path = workspace::worktree_path(base_dir, name);
    let branch = git::worktree_branch(&worktree_path);
    let ab = git::ahead_behind(&worktree_path);
    let has_upstream = ab.is_some();
    let diff = git::diff_stat(&worktree_path);

    let mut parts: Vec<String> = Vec::new();

    match diff {
        Some((0, 0)) => parts.push(format!("{}", theme::green("clean"))),
        Some((added, removed)) => {
            if added > 0 {
                parts.push(format!("{}", theme::green(&format!("+{}", added))));
            }
            if removed > 0 {
                parts.push(format!("{}", theme::red(&format!("-{}", removed))));
            }
        }
        None => {}
    }

    let mut icon = if has_upstream {
        format!("{}", theme::teal(""))
    } else {
        format!("{}", theme::grey(""))
    };

    if gh_available {
        match github::pr_for_branch(&config.repo, &branch) {
            Ok(Some(pr)) => {
                icon = format!(
                    "{}",
                    match &pr.check_status {
                        Some(github::CheckStatus::Passing) => theme::green("󱓏"),
                        Some(github::CheckStatus::Failing) => theme::red("󱓌"),
                        Some(github::CheckStatus::Pending) => theme::yellow("󱓎"),
                        None => theme::light_blue(""),
                    }
                );
                let pr_str = if pr.state == "MERGED" {
                    format!("{}", theme::purple(&format!(" #{}", pr.number)))
                } else {
                    format!("{}", theme::purple(&format!("PR #{}", pr.number)))
                };
                parts.push(pr_str);
            }
            Ok(None) => {}
            Err(e) if !e.contains("no git remotes") => {
                parts.push(format!("{}", theme::red(&format!("gh: {}", e))));
            }
            Err(_) => {}
        }
    }

    WorkspaceStatus {
        name: name.to_string(),
        repo: git::repo_name(&config.repo),
        icon,
        branch,
        status: parts.join("  "),
    }
}

pub fn status(base_dir: &Path, porcelain: bool) -> Result<()> {
    let workspaces = workspace::list_all(base_dir)?;

    if workspaces.is_empty() {
        if !porcelain {
            println!("no workspaces");
        }
        return Ok(());
    }

    let gh_available = github::is_available();
    if !gh_available && !porcelain {
        eprintln!("Install gh for PR details");
    }

    let entries: Vec<WorkspaceStatus> = std::thread::scope(|s| {
        let handles: Vec<_> = workspaces
            .iter()
            .map(|(name, config)| {
                s.spawn(|| gather_workspace_status(base_dir, name, config, gh_available))
            })
            .collect();
        let mut entries: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        entries.sort_by(|a, b| a.repo.cmp(&b.repo));
        entries
    });

    let name_width = entries.iter().map(|e| e.name.len()).max().unwrap().max(9);
    let branch_width = entries
        .iter()
        .map(|e| e.branch.len() + 2)
        .max()
        .unwrap()
        .max(6);

    if porcelain {
        let repo_width = entries.iter().map(|e| e.repo.len()).max().unwrap();
        for entry in &entries {
            println!(
                " {:<name_width$}   {:<repo_width$}  {} {:<w$}  {}",
                entry.name,
                entry.repo,
                entry.icon,
                entry.branch,
                entry.status,
                w = branch_width - 2,
            );
        }
        return Ok(());
    }

    let header = format!(
        " {:<name_width$}  {:<branch_width$}  Status ",
        "Workspace", "Branch"
    );
    println!("{}", theme::header(&header));

    let mut grouped: Vec<(String, Vec<&WorkspaceStatus>)> = Vec::new();
    for entry in &entries {
        if let Some(group) = grouped.last_mut().filter(|(repo, _)| *repo == entry.repo) {
            group.1.push(entry);
        } else {
            grouped.push((entry.repo.clone(), vec![entry]));
        }
    }

    for (i, (repo, group)) in grouped.iter().enumerate() {
        if i > 0 {
            println!();
        }
        println!(" {}", theme::blue(repo));
        for entry in group {
            println!(
                " {}  {} {}  {}",
                format!("{:<name_width$}", entry.name),
                entry.icon,
                format!("{:<w$}", entry.branch, w = branch_width - 2),
                entry.status,
            );
        }
    }

    Ok(())
}
