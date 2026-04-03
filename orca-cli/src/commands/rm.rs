use anyhow::{Result, bail};
use std::path::Path;

use crate::{git, setup, theme, workspace};
use crate::setup::ScriptContext;

pub fn rm(base_dir: &Path, names: &[String], no_script: bool) -> Result<()> {
    let resolved: Vec<String> = if names.is_empty() {
        match workspace::detect_current(base_dir) {
            Some(name) => vec![name],
            None => bail!("not in a workspace and no workspace names given"),
        }
    } else {
        names.to_vec()
    };

    for name in &resolved {
        let config = workspace::load(base_dir, name)?;
        let worktree_path = workspace::worktree_path(base_dir, name);

        if !no_script && worktree_path.exists() {
            let branch_name = git::worktree_branch(&worktree_path);
            setup::run_teardown_scripts(
                base_dir,
                &config.repo,
                &ScriptContext {
                    workspace_name: name,
                    branch_name: &branch_name,
                    workspace_path: &worktree_path,
                },
            );
        }

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
