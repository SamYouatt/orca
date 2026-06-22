use std::path::Path;
use std::process::{Command, Output};

use tempfile::tempdir;

mod common;

use common::setup_test_repo;

fn run_orca(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_orca"))
        .args(args)
        .env("HOME", home)
        .current_dir(cwd)
        .output()
        .unwrap()
}

#[test]
fn test_version_command_prints_package_version() {
    let cwd = tempdir().unwrap();
    let home_dir = tempdir().unwrap();

    let output = run_orca(home_dir.path(), cwd.path(), &["version"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\n", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[test]
fn test_issue_cli_stdout_stderr_contracts() {
    let repo_dir = setup_test_repo();
    let home_dir = tempdir().unwrap();

    let created = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "create", "--title", "CLI issue", "--body", "body"],
    );
    assert!(created.status.success());
    assert_eq!(String::from_utf8(created.stdout).unwrap(), "0000\n");
    assert_eq!(String::from_utf8(created.stderr).unwrap(), "");

    let blocker = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "create", "--title", "Blocker"],
    );
    assert!(blocker.status.success());
    assert_eq!(String::from_utf8(blocker.stdout).unwrap(), "0001\n");

    let blocked = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "block", "0000", "0001"],
    );
    assert!(blocked.status.success());
    assert_eq!(String::from_utf8(blocked.stdout).unwrap(), "");
    assert_eq!(String::from_utf8(blocked.stderr).unwrap(), "");

    let shown_json = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "show", "0000", "--json"],
    );
    assert!(shown_json.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&shown_json.stdout).unwrap();
    assert_eq!(parsed["id"], "0000");
    assert_eq!(parsed["title"], "CLI issue");
    assert_eq!(parsed["blockers"], serde_json::json!(["0001"]));

    let missing = run_orca(home_dir.path(), repo_dir.path(), &["issue", "show", "9999"]);
    assert!(!missing.status.success());
    assert_eq!(String::from_utf8(missing.stdout).unwrap(), "");
    assert!(
        String::from_utf8(missing.stderr)
            .unwrap()
            .contains("issue 9999 not found")
    );
}

#[test]
fn test_issue_cli_blocker_noops_are_reported_as_errors() {
    let repo_dir = setup_test_repo();
    let home_dir = tempdir().unwrap();

    let target = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "create", "--title", "Target"],
    );
    assert!(target.status.success());
    let blocker = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "create", "--title", "Blocker"],
    );
    assert!(blocker.status.success());

    let blocked = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "block", "0000", "0001"],
    );
    assert!(blocked.status.success());

    let duplicate = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "block", "0000", "0001"],
    );
    assert!(!duplicate.status.success());
    assert_eq!(String::from_utf8(duplicate.stdout).unwrap(), "");
    assert!(
        String::from_utf8(duplicate.stderr)
            .unwrap()
            .contains("issue 0001 is already a blocker")
    );

    let unblocked = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "unblock", "0000", "0001"],
    );
    assert!(unblocked.status.success());

    let missing_edge = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "unblock", "0000", "0001"],
    );
    assert!(!missing_edge.status.success());
    assert_eq!(String::from_utf8(missing_edge.stdout).unwrap(), "");
    assert!(
        String::from_utf8(missing_edge.stderr)
            .unwrap()
            .contains("issue 0001 is not a blocker")
    );
}

#[test]
fn test_issue_cli_blocker_validation_takes_precedence_over_noop_edges() {
    let repo_dir = setup_test_repo();
    let home_dir = tempdir().unwrap();

    let created = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "create", "--title", "Target"],
    );
    assert!(created.status.success());

    let missing_duplicate = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "block", "0000", "9999", "9999"],
    );
    assert!(!missing_duplicate.status.success());
    assert!(
        String::from_utf8(missing_duplicate.stderr)
            .unwrap()
            .contains("issue 9999 not found")
    );

    let self_duplicate = run_orca(
        home_dir.path(),
        repo_dir.path(),
        &["issue", "block", "0000", "0000", "0000"],
    );
    assert!(!self_duplicate.status.success());
    assert!(
        String::from_utf8(self_duplicate.stderr)
            .unwrap()
            .contains("issue 0000 cannot block itself")
    );
}
