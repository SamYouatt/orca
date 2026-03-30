use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub repo: PathBuf,
    pub created: DateTime<Utc>,
}

pub fn config_dir(base_dir: &Path) -> PathBuf {
    base_dir.join("config")
}

pub fn config_path(base_dir: &Path, name: &str) -> PathBuf {
    config_dir(base_dir).join(format!("{}.json", name))
}

pub fn worktree_path(base_dir: &Path, name: &str) -> PathBuf {
    base_dir.join("workspaces").join(name)
}

pub fn save(base_dir: &Path, name: &str, config: &WorkspaceConfig) -> Result<()> {
    let path = config_path(base_dir, name);
    fs::create_dir_all(path.parent().unwrap())?;
    let json = serde_json::to_string_pretty(config)?;
    fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn load(base_dir: &Path, name: &str) -> Result<WorkspaceConfig> {
    let path = config_path(base_dir, name);
    let contents =
        fs::read_to_string(&path).with_context(|| format!("workspace '{}' not found", name))?;
    let config: WorkspaceConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

pub fn list_all(base_dir: &Path) -> Result<Vec<(String, WorkspaceConfig)>> {
    let dir = config_dir(base_dir);
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let contents = fs::read_to_string(&path)?;
            if let Ok(config) = serde_json::from_str::<WorkspaceConfig>(&contents) {
                results.push((name, config));
            }
        }
    }
    results.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
}

pub fn delete(base_dir: &Path, name: &str) -> Result<()> {
    let config = config_path(base_dir, name);
    if config.exists() {
        fs::remove_file(&config)?;
    }

    let worktree = worktree_path(base_dir, name);
    if worktree.exists() {
        fs::remove_dir_all(&worktree)?;
    }

    Ok(())
}

pub fn detect_current(base_dir: &Path) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let workspaces_dir = base_dir.join("workspaces");
    let rel = cwd.strip_prefix(&workspaces_dir).ok()?;
    let name = rel.components().next()?;
    let name = name.as_os_str().to_string_lossy().to_string();
    if exists(base_dir, &name) {
        Some(name)
    } else {
        None
    }
}

pub fn exists(base_dir: &Path, name: &str) -> bool {
    config_path(base_dir, name).exists()
}

pub fn resolve_unique_name(base_dir: &Path, candidate: &str) -> String {
    if !exists(base_dir, candidate) {
        return candidate.to_string();
    }

    let mut n = 1;
    loop {
        let name = format!("{}-{}", candidate, n);
        if !exists(base_dir, &name) {
            return name;
        }
        n += 1;
    }
}
