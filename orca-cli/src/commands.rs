use std::path::Path;

use anyhow::Result;
use chrono::Utc;

use crate::{
    git, github, names, theme,
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
        "Created workspace {} on branch {} at {}",
        theme::blue_bold(&name),
        theme::purple(branch),
        theme::blue(&worktree_path.display().to_string())
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

    let header = format!(
        " 󰂽 {:<name_width$}  {:<rb_width$}  {:<16} ",
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
        format!("{}", theme::teal(""))
    } else {
        format!("{}", theme::grey(""))
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
                        None => theme::light_blue(""),
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
                " {:<name_width$}   {:<repo_width$}  {} {:<w$}  {}",
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
