use std::path::PathBuf;

pub mod collection;
pub mod commands;
pub mod config;
pub mod git;
pub mod github;
pub mod names;
pub mod setup;
pub mod sync;
pub mod theme;
pub mod workspace;

pub fn base_dir() -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".orca")
}
