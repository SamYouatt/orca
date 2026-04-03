use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::names::Rarity;

#[derive(Debug, Serialize, Deserialize)]
pub struct CatchRecord {
    pub rarity: Rarity,
    pub caught_at: DateTime<Utc>,
    pub repo: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Collection {
    pub catches: HashMap<String, CatchRecord>,
}

fn collection_path(base_dir: &Path) -> PathBuf {
    base_dir.join("collection.json")
}

pub fn load(base_dir: &Path) -> Collection {
    let path = collection_path(base_dir);
    if !path.exists() {
        return Collection::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(base_dir: &Path, collection: &Collection) -> Result<()> {
    let path = collection_path(base_dir);
    fs::create_dir_all(path.parent().unwrap())?;
    let json = serde_json::to_string_pretty(collection)?;
    fs::write(&path, json)?;
    Ok(())
}

pub fn record_catch(base_dir: &Path, name: &str, rarity: Rarity, repo: &str) -> bool {
    let mut collection = load(base_dir);
    if collection.catches.contains_key(name) {
        return false;
    }
    collection.catches.insert(
        name.to_string(),
        CatchRecord {
            rarity,
            caught_at: Utc::now(),
            repo: repo.to_string(),
        },
    );
    if let Err(e) = save(base_dir, &collection) {
        eprintln!("warning: failed to save collection: {}", e);
    }
    true
}
