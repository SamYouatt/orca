use std::process::Command;

use serial_test::serial;
use tempfile::tempdir;

use orca::{commands, workspace};

fn setup_test_repo() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    dir
}

fn git_branches(repo: &std::path::Path) -> String {
    let output = Command::new("git")
        .args(["-C", &repo.display().to_string(), "branch", "--list"])
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
#[serial]
fn test_full_lifecycle() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    std::env::set_current_dir(repo_dir.path()).unwrap();

    commands::new(orca_dir.path(), None).unwrap();

    let workspaces = workspace::list_all(orca_dir.path()).unwrap();
    assert_eq!(workspaces.len(), 1);

    let (name, config) = &workspaces[0];

    let expected_worktree = workspace::worktree_path(orca_dir.path(), name);
    assert!(expected_worktree.exists());
    assert!(workspace::exists(orca_dir.path(), name));
    assert_eq!(
        config.repo.canonicalize().unwrap(),
        repo_dir.path().canonicalize().unwrap()
    );

    let branches = git_branches(repo_dir.path());
    assert!(branches.contains(name.as_str()));

    assert!(!workspace::config_path(orca_dir.path(), name).starts_with(&expected_worktree));

    commands::rm(orca_dir.path(), &[name.clone()]).unwrap();

    assert!(!workspace::exists(orca_dir.path(), name));
    assert!(!expected_worktree.exists());
    assert!(workspace::list_all(orca_dir.path()).unwrap().is_empty());
}

#[test]
#[serial]
fn test_rm_with_missing_worktree() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    std::env::set_current_dir(repo_dir.path()).unwrap();

    commands::new(orca_dir.path(), None).unwrap();

    let workspaces = workspace::list_all(orca_dir.path()).unwrap();
    let name = workspaces[0].0.clone();
    let worktree = workspace::worktree_path(orca_dir.path(), &name);

    std::fs::remove_dir_all(&worktree).unwrap();

    commands::rm(orca_dir.path(), &[name.clone()]).unwrap();
    assert!(!workspace::exists(orca_dir.path(), &name));
}

#[test]
fn test_name_collision() {
    let orca_dir = tempdir().unwrap();

    assert_eq!(
        workspace::resolve_unique_name(orca_dir.path(), "marlin"),
        "marlin"
    );

    let config = workspace::WorkspaceConfig {
        repo: "/tmp/fake".into(),
        created: chrono::Utc::now(),
    };
    workspace::save(orca_dir.path(), "marlin", &config).unwrap();

    assert_eq!(
        workspace::resolve_unique_name(orca_dir.path(), "marlin"),
        "marlin-1"
    );

    workspace::save(orca_dir.path(), "marlin-1", &config).unwrap();

    assert_eq!(
        workspace::resolve_unique_name(orca_dir.path(), "marlin"),
        "marlin-2"
    );
}

#[test]
#[serial]
fn test_new_with_custom_branch() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    std::env::set_current_dir(repo_dir.path()).unwrap();

    commands::new(orca_dir.path(), Some("feat/my-feature")).unwrap();

    let workspaces = workspace::list_all(orca_dir.path()).unwrap();
    assert_eq!(workspaces.len(), 1);

    let (name, _) = &workspaces[0];

    let branches = git_branches(repo_dir.path());
    assert!(
        !branches.contains(name.as_str()),
        "branch should not match workspace name"
    );
    assert!(
        branches.contains("feat/my-feature"),
        "custom branch should exist"
    );
}

#[test]
#[serial]
fn test_new_outside_git_repo() {
    let not_a_repo = tempdir().unwrap();
    let orca_dir = tempdir().unwrap();

    std::env::set_current_dir(not_a_repo.path()).unwrap();

    let result = commands::new(orca_dir.path(), None);
    assert!(result.is_err());
}
