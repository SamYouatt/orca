use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

use serial_test::serial;
use tempfile::tempdir;

use orca::sync::{self, PendingSide, Side, SyncState};
use orca::{commands, workspace};

mod common;

use common::setup_test_repo;

fn git_branches(repo: &std::path::Path) -> String {
    let output = Command::new("git")
        .args(["-C", &repo.display().to_string(), "branch", "--list"])
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
#[serial]
fn test_issue_create_show_and_repo_scoped_ids() {
    let repo_a = setup_test_repo();
    let repo_b = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    std::env::set_current_dir(repo_a.path()).unwrap();
    let first = commands::issue::create(orca_dir.path(), None, "First issue", "").unwrap();
    let second = commands::issue::create(
        orca_dir.path(),
        Some(repo_a.path()),
        "Second issue",
        "line one\nline two",
    )
    .unwrap();

    assert_eq!(first, "0000");
    assert_eq!(second, "0001");

    let shown = commands::issue::show(orca_dir.path(), Some(repo_a.path()), "0001").unwrap();
    assert!(shown.contains("id: 0001"));
    assert!(shown.contains("title: Second issue"));
    assert!(shown.contains("status: todo"));
    assert!(shown.contains("repo: "));
    assert!(shown.ends_with("line one\nline two"));

    let repo_b_first =
        commands::issue::create(orca_dir.path(), Some(repo_b.path()), "Repo B issue", "").unwrap();
    assert_eq!(repo_b_first, "0000");
}

#[test]
fn test_issue_create_allocates_unique_ids_under_contention() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();
    let workers = 16;
    let barrier = Arc::new(Barrier::new(workers));

    let handles = (0..workers)
        .map(|index| {
            let barrier = Arc::clone(&barrier);
            let base_dir = orca_dir.path().to_path_buf();
            let repo = repo_dir.path().to_path_buf();

            thread::spawn(move || {
                barrier.wait();
                commands::issue::create(&base_dir, Some(&repo), &format!("Issue {index}"), "")
            })
        })
        .collect::<Vec<_>>();

    let mut ids = handles
        .into_iter()
        .map(|handle| handle.join().unwrap().unwrap())
        .collect::<Vec<_>>();
    ids.sort();

    assert_eq!(
        ids,
        (0..workers)
            .map(|id| format!("{id:04}"))
            .collect::<Vec<_>>()
    );
}

#[test]
#[serial]
fn test_issue_show_missing_and_no_repo_errors() {
    let repo_dir = setup_test_repo();
    let not_a_repo = tempdir().unwrap();
    let orca_dir = tempdir().unwrap();

    let missing =
        commands::issue::show(orca_dir.path(), Some(repo_dir.path()), "0000").unwrap_err();
    assert!(missing.to_string().contains("issue 0000 not found"));

    std::env::set_current_dir(not_a_repo.path()).unwrap();
    let no_repo = commands::issue::create(orca_dir.path(), None, "No repo", "").unwrap_err();
    assert!(
        no_repo
            .to_string()
            .contains("could not resolve git repository")
    );
}

#[test]
#[serial]
fn test_issue_list_text_is_repo_scoped_sorted_and_compact() {
    let repo_a = setup_test_repo();
    let repo_b = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    commands::issue::create(orca_dir.path(), Some(repo_a.path()), "Second title", "").unwrap();
    commands::issue::create(orca_dir.path(), Some(repo_a.path()), "First title", "").unwrap();
    commands::issue::create(orca_dir.path(), Some(repo_b.path()), "Other repo", "").unwrap();

    let listed =
        commands::issue::list(orca_dir.path(), Some(repo_a.path()), &[], None, false).unwrap();

    assert_eq!(
        listed,
        "0000  todo  Second title  blockers: -\n0001  todo  First title  blockers: -"
    );
}

#[test]
#[serial]
fn test_issue_list_uses_primary_repo_scope_from_linked_worktree() {
    let repo_dir = setup_test_repo();
    let codex_dir = tempdir().unwrap();
    let orca_dir = tempdir().unwrap();
    let worktree = codex_dir
        .path()
        .join("worktrees")
        .join("243c")
        .join("issue-scope");
    std::fs::create_dir_all(worktree.parent().unwrap()).unwrap();

    Command::new("git")
        .args([
            "-C",
            &repo_dir.path().display().to_string(),
            "worktree",
            "add",
            "-b",
            "issue-scope",
            &worktree.display().to_string(),
        ])
        .output()
        .unwrap();

    commands::issue::create(
        orca_dir.path(),
        Some(repo_dir.path()),
        "Main checkout issue",
        "",
    )
    .unwrap();

    std::env::set_current_dir(&worktree).unwrap();
    let listed = commands::issue::list(orca_dir.path(), None, &[], None, false).unwrap();

    assert_eq!(listed, "0000  todo  Main checkout issue  blockers: -");
}

#[test]
#[serial]
fn test_issue_list_json_show_json_and_repeated_status_filters() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Alpha", "body").unwrap();
    commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Beta", "").unwrap();

    let listed = commands::issue::list(
        orca_dir.path(),
        Some(repo_dir.path()),
        &["todo".into()],
        None,
        true,
    )
    .unwrap();
    let listed_json: serde_json::Value = serde_json::from_str(&listed).unwrap();

    assert_eq!(listed_json[0]["id"], "0000");
    assert_eq!(listed_json[0]["status"], "todo");
    assert_eq!(listed_json[0]["title"], "Alpha");
    assert_eq!(listed_json[0]["blockers"], serde_json::json!([]));
    assert_eq!(listed_json[1]["id"], "0001");
    assert!(listed_json[0]["repo_path"].is_null());

    let empty = commands::issue::list(
        orca_dir.path(),
        Some(repo_dir.path()),
        &["done".into()],
        None,
        false,
    )
    .unwrap();
    assert_eq!(empty, "");

    let shown = commands::issue::show_json(orca_dir.path(), Some(repo_dir.path()), "0000").unwrap();
    let shown_json: serde_json::Value = serde_json::from_str(&shown).unwrap();

    assert_eq!(shown_json["id"], "0000");
    assert_eq!(
        shown_json["repo_path"],
        repo_dir
            .path()
            .canonicalize()
            .unwrap()
            .display()
            .to_string()
    );
    assert_eq!(shown_json["body"], "body");
    assert_eq!(shown_json["blockers"], serde_json::json!([]));
    assert_eq!(shown_json["blocked"], serde_json::json!([]));
}

#[test]
#[serial]
fn test_issue_dependency_graph_and_blocked_by_filter() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let blocker =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Foundation", "").unwrap();
    let downstream = commands::issue::create(
        orca_dir.path(),
        Some(repo_dir.path()),
        "Build on foundation",
        "details",
    )
    .unwrap();

    commands::issue::block(
        orca_dir.path(),
        Some(repo_dir.path()),
        &downstream,
        &[blocker.as_str()],
    )
    .unwrap();

    let shown = commands::issue::show(orca_dir.path(), Some(repo_dir.path()), &downstream).unwrap();
    assert!(shown.contains("blockers: 0000"));
    assert!(shown.contains("blocked: -"));

    let blocker_json =
        commands::issue::show_json(orca_dir.path(), Some(repo_dir.path()), &blocker).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&blocker_json).unwrap();
    assert_eq!(parsed["id"], "0000");
    assert_eq!(parsed["blocked"], serde_json::json!(["0001"]));

    let blocked_by = commands::issue::list(
        orca_dir.path(),
        Some(repo_dir.path()),
        &[],
        Some(&blocker),
        true,
    )
    .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&blocked_by).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 1);
    assert_eq!(parsed[0]["id"], "0001");
    assert_eq!(parsed[0]["blockers"], serde_json::json!(["0000"]));
}

#[test]
#[serial]
fn test_issue_dependency_rejects_invalid_blockers() {
    let repo_a = setup_test_repo();
    let repo_b = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let first = commands::issue::create(orca_dir.path(), Some(repo_a.path()), "First", "").unwrap();
    commands::issue::create(orca_dir.path(), Some(repo_b.path()), "Other repo 0", "").unwrap();
    commands::issue::create(orca_dir.path(), Some(repo_b.path()), "Other repo 1", "").unwrap();
    commands::issue::create(orca_dir.path(), Some(repo_b.path()), "Other repo 2", "").unwrap();
    let repo_b_issue =
        commands::issue::create(orca_dir.path(), Some(repo_b.path()), "Other repo 3", "").unwrap();

    let self_dependency = commands::issue::block(
        orca_dir.path(),
        Some(repo_a.path()),
        &first,
        &[first.as_str()],
    )
    .unwrap_err();
    assert!(self_dependency.to_string().contains("cannot block itself"));

    let missing = commands::issue::block(orca_dir.path(), Some(repo_a.path()), &first, &["9999"])
        .unwrap_err();
    assert!(missing.to_string().contains("issue 9999 not found"));

    let cross_repo = commands::issue::block(
        orca_dir.path(),
        Some(repo_a.path()),
        &first,
        &[repo_b_issue.as_str()],
    )
    .unwrap_err();
    assert!(cross_repo.to_string().contains("issue 0003 not found"));
}

#[test]
#[serial]
fn test_issue_dependency_cycle_failure_is_atomic() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let first =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "First", "").unwrap();
    let second =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Second", "").unwrap();
    let third =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Third", "").unwrap();

    commands::issue::block(
        orca_dir.path(),
        Some(repo_dir.path()),
        &second,
        &[first.as_str()],
    )
    .unwrap();
    commands::issue::block(
        orca_dir.path(),
        Some(repo_dir.path()),
        &third,
        &[second.as_str()],
    )
    .unwrap();

    let cycle = commands::issue::block(
        orca_dir.path(),
        Some(repo_dir.path()),
        &first,
        &[third.as_str()],
    )
    .unwrap_err();
    assert!(cycle.to_string().contains("cycle"));

    let still_unblocked =
        commands::issue::show(orca_dir.path(), Some(repo_dir.path()), &first).unwrap();
    assert!(still_unblocked.contains("blockers: -"));
}

#[test]
#[serial]
fn test_issue_unblock_and_missing_blocked_by_filter() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let blocker =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Blocker", "").unwrap();
    let blocked =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Blocked", "").unwrap();

    commands::issue::block(
        orca_dir.path(),
        Some(repo_dir.path()),
        &blocked,
        &[blocker.as_str()],
    )
    .unwrap();
    commands::issue::unblock(
        orca_dir.path(),
        Some(repo_dir.path()),
        &blocked,
        &[blocker.as_str()],
    )
    .unwrap();

    let shown = commands::issue::show(orca_dir.path(), Some(repo_dir.path()), &blocked).unwrap();
    assert!(shown.contains("blockers: -"));

    let missing_filter = commands::issue::list(
        orca_dir.path(),
        Some(repo_dir.path()),
        &[],
        Some("9999"),
        false,
    )
    .unwrap_err();
    assert!(missing_filter.to_string().contains("issue 9999 not found"));
}

#[test]
#[serial]
fn test_issue_block_and_unblock_reject_noop_edges() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let blocker =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Blocker", "").unwrap();
    let blocked =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Blocked", "").unwrap();

    commands::issue::block(
        orca_dir.path(),
        Some(repo_dir.path()),
        &blocked,
        &[blocker.as_str()],
    )
    .unwrap();

    let duplicate = commands::issue::block(
        orca_dir.path(),
        Some(repo_dir.path()),
        &blocked,
        &[blocker.as_str()],
    )
    .unwrap_err();
    assert!(duplicate.to_string().contains("already a blocker"));

    let shown = commands::issue::show(orca_dir.path(), Some(repo_dir.path()), &blocked).unwrap();
    assert!(shown.contains("blockers: 0000"));

    commands::issue::unblock(
        orca_dir.path(),
        Some(repo_dir.path()),
        &blocked,
        &[blocker.as_str()],
    )
    .unwrap();

    let missing = commands::issue::unblock(
        orca_dir.path(),
        Some(repo_dir.path()),
        &blocked,
        &[blocker.as_str()],
    )
    .unwrap_err();
    assert!(missing.to_string().contains("not a blocker"));

    let shown = commands::issue::show(orca_dir.path(), Some(repo_dir.path()), &blocked).unwrap();
    assert!(shown.contains("blockers: -"));
}

#[test]
#[serial]
fn test_issue_block_and_unblock_validate_blocker_arguments_before_store_edges() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let empty =
        commands::issue::block(orca_dir.path(), Some(repo_dir.path()), "9999", &[]).unwrap_err();
    assert!(
        empty
            .to_string()
            .contains("at least one blocker id is required")
    );

    let invalid = commands::issue::unblock(
        orca_dir.path(),
        Some(repo_dir.path()),
        "9999",
        &["not-an-id"],
    )
    .unwrap_err();
    assert!(invalid.to_string().contains("invalid issue id"));
}

#[test]
#[serial]
fn test_issue_update_patches_fields_and_clears_body() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let issue = commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Original", "body")
        .unwrap();

    commands::issue::update(
        orca_dir.path(),
        Some(repo_dir.path()),
        &issue,
        commands::issue::IssueUpdate {
            title: Some("Renamed".into()),
            status: Some("doing".into()),
            body: None,
            blockers: commands::issue::BlockerUpdate::Unchanged,
        },
    )
    .unwrap();

    let shown = commands::issue::show(orca_dir.path(), Some(repo_dir.path()), &issue).unwrap();
    assert!(shown.contains("title: Renamed"));
    assert!(shown.contains("status: doing"));
    assert!(shown.ends_with("body"));

    commands::issue::update(
        orca_dir.path(),
        Some(repo_dir.path()),
        &issue,
        commands::issue::IssueUpdate {
            title: None,
            status: None,
            body: Some("".into()),
            blockers: commands::issue::BlockerUpdate::Unchanged,
        },
    )
    .unwrap();

    let shown = commands::issue::show(orca_dir.path(), Some(repo_dir.path()), &issue).unwrap();
    assert!(shown.contains("title: Renamed"));
    assert!(shown.contains("status: doing"));
    assert!(shown.ends_with("\n\n"));
}

#[test]
#[serial]
fn test_issue_update_replaces_adds_and_removes_blockers_atomically() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let first =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "First", "").unwrap();
    let second =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Second", "").unwrap();
    let third =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Third", "").unwrap();
    let target =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Target", "").unwrap();

    commands::issue::update(
        orca_dir.path(),
        Some(repo_dir.path()),
        &target,
        commands::issue::IssueUpdate {
            title: None,
            status: None,
            body: None,
            blockers: commands::issue::BlockerUpdate::Replace(vec![first.clone(), second.clone()]),
        },
    )
    .unwrap();

    let shown = commands::issue::show(orca_dir.path(), Some(repo_dir.path()), &target).unwrap();
    assert!(shown.contains("blockers: 0000,0001"));

    commands::issue::update(
        orca_dir.path(),
        Some(repo_dir.path()),
        &target,
        commands::issue::IssueUpdate {
            title: None,
            status: None,
            body: None,
            blockers: commands::issue::BlockerUpdate::Add(vec![third.clone()]),
        },
    )
    .unwrap();
    commands::issue::update(
        orca_dir.path(),
        Some(repo_dir.path()),
        &target,
        commands::issue::IssueUpdate {
            title: None,
            status: None,
            body: None,
            blockers: commands::issue::BlockerUpdate::Remove(vec![first.clone()]),
        },
    )
    .unwrap();

    let shown = commands::issue::show(orca_dir.path(), Some(repo_dir.path()), &target).unwrap();
    assert!(shown.contains("blockers: 0001,0002"));

    commands::issue::block(
        orca_dir.path(),
        Some(repo_dir.path()),
        &first,
        &[target.as_str()],
    )
    .unwrap();
    let cycle = commands::issue::update(
        orca_dir.path(),
        Some(repo_dir.path()),
        &target,
        commands::issue::IssueUpdate {
            title: Some("Should not apply".into()),
            status: None,
            body: None,
            blockers: commands::issue::BlockerUpdate::Replace(vec![first.clone()]),
        },
    )
    .unwrap_err();
    assert!(cycle.to_string().contains("cycle"));

    let shown = commands::issue::show(orca_dir.path(), Some(repo_dir.path()), &target).unwrap();
    assert!(shown.contains("title: Target"));
    assert!(shown.contains("blockers: 0001,0002"));
}

#[test]
#[serial]
fn test_issue_update_rejects_empty_and_noop_mutations() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let blocker =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Blocker", "").unwrap();
    let target =
        commands::issue::create(orca_dir.path(), Some(repo_dir.path()), "Target", "body").unwrap();

    let empty = commands::issue::update(
        orca_dir.path(),
        Some(repo_dir.path()),
        &target,
        commands::issue::IssueUpdate {
            title: None,
            status: None,
            body: None,
            blockers: commands::issue::BlockerUpdate::Unchanged,
        },
    )
    .unwrap_err();
    assert!(
        empty
            .to_string()
            .contains("at least one update is required")
    );

    let noop_fields = commands::issue::update(
        orca_dir.path(),
        Some(repo_dir.path()),
        &target,
        commands::issue::IssueUpdate {
            title: Some("Target".into()),
            status: None,
            body: Some("body".into()),
            blockers: commands::issue::BlockerUpdate::Unchanged,
        },
    )
    .unwrap_err();
    assert!(noop_fields.to_string().contains("no changes to apply"));

    let noop_remove = commands::issue::update(
        orca_dir.path(),
        Some(repo_dir.path()),
        &target,
        commands::issue::IssueUpdate {
            title: None,
            status: None,
            body: None,
            blockers: commands::issue::BlockerUpdate::Remove(vec![blocker.clone()]),
        },
    )
    .unwrap_err();
    assert!(noop_remove.to_string().contains("not a blocker"));
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

    commands::rm(orca_dir.path(), std::slice::from_ref(name), false).unwrap();

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

    commands::rm(orca_dir.path(), std::slice::from_ref(&name), false).unwrap();
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
    std::fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
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

#[test]
#[serial]
fn test_rm_runs_global_teardown_script() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let marker = orca_dir.path().join("global-teardown-ran");
    write_script(
        &orca_dir.path().join("teardown.sh"),
        &format!("touch {}", marker.display()),
    );

    std::fs::write(
        orca_dir.path().join("settings.json"),
        r#"{ "teardown": { "script": "teardown.sh" } }"#,
    )
    .unwrap();

    std::env::set_current_dir(repo_dir.path()).unwrap();
    commands::new(orca_dir.path(), None, true).unwrap();

    let workspaces = workspace::list_all(orca_dir.path()).unwrap();
    let name = workspaces[0].0.clone();

    commands::rm(orca_dir.path(), &[name], false).unwrap();

    assert!(marker.exists(), "global teardown script should have run");
}

#[test]
#[serial]
fn test_rm_runs_project_teardown_script() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let marker = orca_dir.path().join("project-teardown-ran");
    write_script(
        &repo_dir.path().join("teardown.sh"),
        &format!("touch {}", marker.display()),
    );

    std::fs::write(
        repo_dir.path().join("orca.json"),
        r#"{ "teardown": { "script": "teardown.sh" } }"#,
    )
    .unwrap();

    std::env::set_current_dir(repo_dir.path()).unwrap();
    commands::new(orca_dir.path(), None, true).unwrap();

    let workspaces = workspace::list_all(orca_dir.path()).unwrap();
    let name = workspaces[0].0.clone();

    commands::rm(orca_dir.path(), &[name], false).unwrap();

    assert!(marker.exists(), "project teardown script should have run");
}

#[test]
#[serial]
fn test_project_teardown_runs_before_global_teardown() {
    let repo_dir = setup_test_repo();
    let orca_dir = tempdir().unwrap();

    let log = orca_dir.path().join("teardown-order.log");

    write_script(
        &orca_dir.path().join("teardown.sh"),
        &format!("echo global >> {}", log.display()),
    );
    std::fs::write(
        orca_dir.path().join("settings.json"),
        r#"{ "teardown": { "script": "teardown.sh" } }"#,
    )
    .unwrap();

    write_script(
        &repo_dir.path().join("teardown.sh"),
        &format!("echo project >> {}", log.display()),
    );
    std::fs::write(
        repo_dir.path().join("orca.json"),
        r#"{ "teardown": { "script": "teardown.sh" } }"#,
    )
    .unwrap();

    std::env::set_current_dir(repo_dir.path()).unwrap();
    commands::new(orca_dir.path(), None, true).unwrap();

    let workspaces = workspace::list_all(orca_dir.path()).unwrap();
    let name = workspaces[0].0.clone();

    commands::rm(orca_dir.path(), &[name], false).unwrap();

    let contents = std::fs::read_to_string(&log).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines, vec!["project", "global"]);
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

#[test]
fn test_initial_scan_syncs_new_worktree_file() {
    let root = tempdir().unwrap();
    let worktree = tempdir().unwrap();

    std::fs::write(root.path().join("shared.txt"), "same").unwrap();
    std::fs::write(worktree.path().join("shared.txt"), "same").unwrap();
    std::fs::write(worktree.path().join("new.txt"), "from worktree").unwrap();

    let state = SyncState::new();
    let root_filter = sync::build_filter(root.path()).unwrap();
    let wt_filter = sync::build_filter(worktree.path()).unwrap();

    let synced = sync::initial_scan(
        &state,
        root.path(),
        worktree.path(),
        &root_filter,
        &wt_filter,
        false,
    );

    assert_eq!(synced.len(), 1);
    assert_eq!(synced[0].0, PathBuf::from("new.txt"));
    assert_eq!(synced[0].1, Side::Worktree);
    assert_eq!(
        std::fs::read_to_string(root.path().join("new.txt")).unwrap(),
        "from worktree"
    );
}

#[test]
fn test_initial_scan_syncs_modified_worktree_file() {
    let root = tempdir().unwrap();
    let worktree = tempdir().unwrap();

    std::fs::write(root.path().join("file.txt"), "original").unwrap();
    std::fs::write(worktree.path().join("file.txt"), "modified").unwrap();

    let state = SyncState::new();
    let root_filter = sync::build_filter(root.path()).unwrap();
    let wt_filter = sync::build_filter(worktree.path()).unwrap();

    let synced = sync::initial_scan(
        &state,
        root.path(),
        worktree.path(),
        &root_filter,
        &wt_filter,
        false,
    );

    assert_eq!(synced.len(), 1);
    assert_eq!(synced[0].1, Side::Worktree);
    assert_eq!(
        std::fs::read_to_string(root.path().join("file.txt")).unwrap(),
        "modified"
    );
}

#[test]
fn test_initial_scan_propagates_worktree_deletion() {
    let root = tempdir().unwrap();
    let worktree = tempdir().unwrap();

    std::fs::write(root.path().join("deleted.txt"), "will be removed").unwrap();
    // file does not exist in worktree

    let state = SyncState::new();
    let root_filter = sync::build_filter(root.path()).unwrap();
    let wt_filter = sync::build_filter(worktree.path()).unwrap();

    let synced = sync::initial_scan(
        &state,
        root.path(),
        worktree.path(),
        &root_filter,
        &wt_filter,
        false,
    );

    assert_eq!(synced.len(), 1);
    assert!(!root.path().join("deleted.txt").exists());
}

#[test]
fn test_initial_scan_skips_identical_files() {
    let root = tempdir().unwrap();
    let worktree = tempdir().unwrap();

    std::fs::write(root.path().join("same.txt"), "identical").unwrap();
    std::fs::write(worktree.path().join("same.txt"), "identical").unwrap();

    let state = SyncState::new();
    let root_filter = sync::build_filter(root.path()).unwrap();
    let wt_filter = sync::build_filter(worktree.path()).unwrap();

    let synced = sync::initial_scan(
        &state,
        root.path(),
        worktree.path(),
        &root_filter,
        &wt_filter,
        false,
    );

    assert!(synced.is_empty());
}
