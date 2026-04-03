use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serial_test::serial;
use tempfile::tempdir;

use orca::sync::{self, PendingSide, Side, SyncState};
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

    commands::new(orca_dir.path(), None, false).unwrap();

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

    commands::new(orca_dir.path(), None, false).unwrap();

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

    commands::new(orca_dir.path(), Some("feat/my-feature"), false).unwrap();

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

    let result = commands::new(orca_dir.path(), None, false);
    assert!(result.is_err());
}

fn write_script(path: &Path, body: &str) {
    std::fs::write(
        path,
        format!("#!/bin/sh\n{body}\n"),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

#[test]
#[serial]
fn test_new_runs_global_setup_script() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let marker = orca_dir.path().join("global-ran");
    write_script(
        &orca_dir.path().join("setup.sh"),
        &format!("touch {}", marker.display()),
    );

    std::fs::write(
        orca_dir.path().join("settings.json"),
        r#"{ "setup": { "script": "setup.sh" } }"#,
    )
    .unwrap();

    std::env::set_current_dir(repo_dir.path()).unwrap();
    commands::new(orca_dir.path(), None, false).unwrap();

    assert!(marker.exists(), "global setup script should have run");
}

#[test]
#[serial]
fn test_new_runs_project_setup_script() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let marker = orca_dir.path().join("project-ran");
    write_script(
        &repo_dir.path().join("setup.sh"),
        &format!("touch {}", marker.display()),
    );

    std::fs::write(
        repo_dir.path().join("orca.json"),
        r#"{ "setup": { "script": "setup.sh" } }"#,
    )
    .unwrap();

    std::env::set_current_dir(repo_dir.path()).unwrap();
    commands::new(orca_dir.path(), None, false).unwrap();

    assert!(marker.exists(), "project setup script should have run");
}

#[test]
#[serial]
fn test_global_setup_runs_before_project_setup() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let log = orca_dir.path().join("order.log");

    write_script(
        &orca_dir.path().join("setup.sh"),
        &format!("echo global >> {}", log.display()),
    );
    std::fs::write(
        orca_dir.path().join("settings.json"),
        r#"{ "setup": { "script": "setup.sh" } }"#,
    )
    .unwrap();

    write_script(
        &repo_dir.path().join("setup.sh"),
        &format!("echo project >> {}", log.display()),
    );
    std::fs::write(
        repo_dir.path().join("orca.json"),
        r#"{ "setup": { "script": "setup.sh" } }"#,
    )
    .unwrap();

    std::env::set_current_dir(repo_dir.path()).unwrap();
    commands::new(orca_dir.path(), None, false).unwrap();

    let contents = std::fs::read_to_string(&log).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines, vec!["global", "project"]);
}

fn past_debounce() -> Instant {
    Instant::now() - std::time::Duration::from_millis(300)
}

#[test]
fn test_sync_root_to_worktree() {
    let root = tempdir().unwrap();
    let worktree = tempdir().unwrap();

    std::fs::write(root.path().join("hello.txt"), "from root").unwrap();

    let state = SyncState::new();
    state.pending.lock().unwrap().insert(
        PathBuf::from("hello.txt"),
        (past_debounce(), PendingSide::One(Side::Root)),
    );

    let root_filter = sync::build_filter(root.path()).unwrap();
    let wt_filter = sync::build_filter(worktree.path()).unwrap();

    let synced = sync::sync_once(
        &state,
        root.path(),
        worktree.path(),
        &root_filter,
        &wt_filter,
        true,
    );

    assert_eq!(synced.len(), 1);
    assert_eq!(synced[0].0, PathBuf::from("hello.txt"));
    assert_eq!(synced[0].1, Side::Root);
    assert_eq!(
        std::fs::read_to_string(worktree.path().join("hello.txt")).unwrap(),
        "from root"
    );
}

#[test]
fn test_sync_worktree_to_root() {
    let root = tempdir().unwrap();
    let worktree = tempdir().unwrap();

    std::fs::write(worktree.path().join("agent.txt"), "from worktree").unwrap();

    let state = SyncState::new();
    state.pending.lock().unwrap().insert(
        PathBuf::from("agent.txt"),
        (past_debounce(), PendingSide::One(Side::Worktree)),
    );

    let root_filter = sync::build_filter(root.path()).unwrap();
    let wt_filter = sync::build_filter(worktree.path()).unwrap();

    let synced = sync::sync_once(
        &state,
        root.path(),
        worktree.path(),
        &root_filter,
        &wt_filter,
        true,
    );

    assert_eq!(synced.len(), 1);
    assert_eq!(synced[0].1, Side::Worktree);
    assert_eq!(
        std::fs::read_to_string(root.path().join("agent.txt")).unwrap(),
        "from worktree"
    );

    assert!(
        state
            .root_written
            .lock()
            .unwrap()
            .contains(&PathBuf::from("agent.txt"))
    );
}

#[test]
fn test_sync_conflict_root_wins() {
    let root = tempdir().unwrap();
    let worktree = tempdir().unwrap();

    std::fs::write(root.path().join("conflict.txt"), "root version").unwrap();
    std::fs::write(worktree.path().join("conflict.txt"), "worktree version").unwrap();

    let state = SyncState::new();
    state.pending.lock().unwrap().insert(
        PathBuf::from("conflict.txt"),
        (past_debounce(), PendingSide::Both),
    );

    let root_filter = sync::build_filter(root.path()).unwrap();
    let wt_filter = sync::build_filter(worktree.path()).unwrap();

    let synced = sync::sync_once(
        &state,
        root.path(),
        worktree.path(),
        &root_filter,
        &wt_filter,
        true,
    );

    assert_eq!(synced.len(), 1);
    assert_eq!(synced[0].1, Side::Root);
    assert_eq!(
        std::fs::read_to_string(worktree.path().join("conflict.txt")).unwrap(),
        "root version"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("conflict.txt")).unwrap(),
        "root version"
    );
}

#[test]
fn test_sync_delete_propagation() {
    let root = tempdir().unwrap();
    let worktree = tempdir().unwrap();

    std::fs::write(worktree.path().join("gone.txt"), "will be deleted").unwrap();

    let state = SyncState::new();
    state.pending.lock().unwrap().insert(
        PathBuf::from("gone.txt"),
        (past_debounce(), PendingSide::One(Side::Root)),
    );

    let root_filter = sync::build_filter(root.path()).unwrap();
    let wt_filter = sync::build_filter(worktree.path()).unwrap();

    sync::sync_once(
        &state,
        root.path(),
        worktree.path(),
        &root_filter,
        &wt_filter,
        true,
    );

    assert!(!worktree.path().join("gone.txt").exists());
}

#[test]
fn test_sync_gitignore_filtering() {
    let root = tempdir().unwrap();

    std::fs::write(root.path().join(".gitignore"), "*.log\ntarget/\n").unwrap();
    std::fs::write(root.path().join("debug.log"), "logs").unwrap();

    let filter = sync::build_filter(root.path()).unwrap();

    assert!(sync::is_ignored(
        &filter,
        root.path(),
        &root.path().join("debug.log")
    ));
    assert!(sync::is_ignored(
        &filter,
        root.path(),
        &root.path().join(".git/config")
    ));
    assert!(!sync::is_ignored(
        &filter,
        root.path(),
        &root.path().join("src/main.rs")
    ));
}

#[test]
fn test_sync_in_flight_prevents_requeue() {
    let root = tempdir().unwrap();
    let worktree = tempdir().unwrap();

    std::fs::write(root.path().join("file.txt"), "original").unwrap();

    let state = SyncState::new();
    let root_filter = sync::build_filter(root.path()).unwrap();
    let wt_filter = sync::build_filter(worktree.path()).unwrap();

    state.pending.lock().unwrap().insert(
        PathBuf::from("file.txt"),
        (past_debounce(), PendingSide::One(Side::Root)),
    );
    sync::sync_once(
        &state,
        root.path(),
        worktree.path(),
        &root_filter,
        &wt_filter,
        true,
    );

    assert!(
        state
            .in_flight
            .lock()
            .unwrap()
            .contains(&worktree.path().join("file.txt")),
        "destination should be in in_flight after copy"
    );

    assert_eq!(
        std::fs::read_to_string(worktree.path().join("file.txt")).unwrap(),
        "original"
    );
}

#[test]
fn test_sync_in_flight_cleared_on_failure() {
    let root = tempdir().unwrap();

    let src = root.path().join("exists.txt");
    std::fs::write(&src, "content").unwrap();
    let dst = PathBuf::from("/nonexistent_root_path/sub/file.txt");

    let state = SyncState::new();
    let result = sync::copy_or_delete(&src, &dst, &state);

    assert!(result.is_err());
    assert!(
        !state.in_flight.lock().unwrap().contains(&dst),
        "in_flight should be cleared on copy failure"
    );
}

#[test]
fn test_sync_pending_side_merge() {
    let side = PendingSide::One(Side::Root);
    assert!(side.has_root());
    assert!(!side.has_worktree());

    let merged = side.merge(Side::Worktree);
    assert_eq!(merged, PendingSide::Both);
    assert!(merged.has_root());
    assert!(merged.has_worktree());

    let same = PendingSide::One(Side::Root).merge(Side::Root);
    assert_eq!(same, PendingSide::One(Side::Root));
}

#[test]
#[serial]
fn test_sync_cleanup_restores_root() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    std::env::set_current_dir(repo_dir.path()).unwrap();

    std::fs::write(repo_dir.path().join("original.txt"), "original content").unwrap();
    Command::new("git")
        .args(["add", "original.txt"])
        .current_dir(repo_dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "add original"])
        .current_dir(repo_dir.path())
        .output()
        .unwrap();

    commands::new(orca_dir.path(), None, false).unwrap();
    let workspaces = workspace::list_all(orca_dir.path()).unwrap();
    let (name, _config) = &workspaces[0];
    let worktree_path = workspace::worktree_path(orca_dir.path(), name);

    std::fs::write(worktree_path.join("original.txt"), "modified by agent").unwrap();

    let state = SyncState::new();
    state.pending.lock().unwrap().insert(
        PathBuf::from("original.txt"),
        (past_debounce(), PendingSide::One(Side::Worktree)),
    );

    let root_filter = sync::build_filter(repo_dir.path()).unwrap();
    let wt_filter = sync::build_filter(&worktree_path).unwrap();

    sync::sync_once(
        &state,
        repo_dir.path(),
        &worktree_path,
        &root_filter,
        &wt_filter,
        true,
    );

    assert_eq!(
        std::fs::read_to_string(repo_dir.path().join("original.txt")).unwrap(),
        "modified by agent"
    );

    let written = state.root_written.lock().unwrap().clone();
    assert!(!written.is_empty());

    let root_str = repo_dir.path().to_string_lossy().to_string();
    let output = Command::new("git")
        .args(["-C", &root_str, "checkout", "--", "original.txt"])
        .output()
        .unwrap();
    assert!(output.status.success());

    assert_eq!(
        std::fs::read_to_string(repo_dir.path().join("original.txt")).unwrap(),
        "original content"
    );
}
