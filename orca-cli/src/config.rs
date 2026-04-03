use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct SetupConfig {
    pub script: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct GlobalSettings {
    pub setup: Option<SetupConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectConfig {
    pub setup: Option<SetupConfig>,
}

pub fn load_global_settings(base_dir: &Path) -> GlobalSettings {
    let path = base_dir.join("settings.json");
    match fs::read_to_string(&path) {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("warning: failed to parse {}: {}", path.display(), e);
                GlobalSettings::default()
            }
        },
        Err(_) => GlobalSettings::default(),
    }
}

pub fn load_project_config(repo: &Path) -> ProjectConfig {
    let path = repo.join("orca.json");
    match fs::read_to_string(&path) {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("warning: failed to parse {}: {}", path.display(), e);
                ProjectConfig::default()
            }
        },
        Err(_) => ProjectConfig::default(),
    }
}

pub fn resolve_script(setup: Option<&SetupConfig>, base: &Path) -> Option<PathBuf> {
    let script = setup?.script.as_deref()?;
    let path = Path::new(script);
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        Some(base.join(path))
    }
}
