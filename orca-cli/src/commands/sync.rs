use anyhow::Result;
use std::path::Path;

use crate::{git, sync, theme, workspace};

pub fn sync(base_dir: &Path, name: Option<&str>, verbose: bool, force: bool) -> Result<()> {
    let name = match name {
        Some(n) => n.to_string(),
        None => workspace::detect_current(base_dir).ok_or_else(|| {
            anyhow::anyhow!(
                "You must either be in a workspace or pass a workspace with the --workspace (-w) flag"
            )
        })?,
    };
    let config = workspace::load(base_dir, &name)?;
    let worktree_path = workspace::worktree_path(base_dir, &name);

    if !force && git::has_uncommitted_changes(&config.repo) {
        anyhow::bail!(
            "root repo has uncommitted changes — commit or stash them first, or use --force to sync anyway"
        );
    }

    println!(
        "  {} syncing {} {} ↔ {}",
        theme::blue_bold("⟳"),
        theme::blue_bold(&name),
        theme::grey(&config.repo.display().to_string()),
        theme::grey(&worktree_path.display().to_string()),
    );
    println!("  {}", theme::grey("ctrl+c to stop"));
    println!();

    sync::run(&config.repo, &worktree_path, verbose)
}
