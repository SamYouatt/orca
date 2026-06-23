use std::process::Command;

use super::types::{CommitOption, DiffSource, FileContents};

pub fn get_default_branch() -> String {
    if let Ok(output) = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()
    {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !s.is_empty() {
            return s.replace("refs/remotes/origin/", "");
        }
    }
    "main".to_string()
}

pub fn get_current_branch() -> String {
    if let Ok(output) = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
    {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() {
            return branch;
        }
    }

    "HEAD".to_string()
}

fn git_diff(args: &[&str]) -> Result<String, String> {
    let mut full_args = args.to_vec();
    // Force standard a/ b/ prefixes regardless of diff.mnemonicPrefix config.
    full_args.push("--find-renames");
    full_args.push("--src-prefix=a/");
    full_args.push("--dst-prefix=b/");
    Command::new("git")
        .args(&full_args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .map_err(|e| e.to_string())
}

pub fn list_branch_commits(default_branch: &str) -> Result<Vec<CommitOption>, String> {
    let base = merge_base(default_branch)?;
    let output = Command::new("git")
        .args([
            "log",
            "--format=%H%x1f%h%x1f%s%x1f%b%x1e",
            &format!("{base}..HEAD"),
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .split('\x1e')
        .filter_map(|record| {
            let record = record.trim_matches('\n');
            if record.is_empty() {
                return None;
            }

            let mut parts = record.splitn(4, '\x1f');
            let sha = parts.next()?.to_string();
            let short_sha = parts.next()?.to_string();
            let subject = parts.next()?.to_string();
            let description = parts
                .next()
                .map(str::trim)
                .filter(|description| !description.is_empty())
                .map(ToString::to_string);
            Some(CommitOption {
                sha,
                short_sha,
                subject,
                description,
            })
        })
        .collect())
}

pub fn selected_commit_option(
    commits: &[CommitOption],
    selected_sha: Option<&str>,
) -> Option<CommitOption> {
    selected_sha.and_then(|selected| {
        commits
            .iter()
            .find(|option| option.sha == selected || option.short_sha == selected)
            .cloned()
    })
}

fn resolve_branch_commit(default_branch: &str, commit: &str) -> Result<CommitOption, String> {
    let commits = list_branch_commits(default_branch)?;
    selected_commit_option(&commits, Some(commit))
        .ok_or_else(|| format!("Commit {commit} is not selectable on this branch"))
}

pub fn run_diff(source: &DiffSource, default_branch: &str) -> (String, String, Option<String>) {
    match source {
        DiffSource::Branch => run_branch_diff(default_branch),
        DiffSource::Commit(commit) => run_commit_diff(default_branch, commit),
        DiffSource::Uncommitted => run_uncommitted_diff(),
    }
}

fn run_branch_diff(default_branch: &str) -> (String, String, Option<String>) {
    let base = match merge_base(default_branch) {
        Ok(base) => base,
        Err(error) => return (String::new(), format!("vs {default_branch}"), Some(error)),
    };

    match git_diff(&["diff", &base]) {
        Ok(mut patch) => {
            if let Ok(untracked) = untracked_file_patches() {
                patch.push_str(&untracked);
            }
            (patch, format!("vs {default_branch}"), None)
        }
        Err(e) => (String::new(), format!("vs {default_branch}"), Some(e)),
    }
}

fn run_commit_diff(default_branch: &str, commit: &str) -> (String, String, Option<String>) {
    let selected = match resolve_branch_commit(default_branch, commit) {
        Ok(commit) => commit,
        Err(error) => return (String::new(), "Commit".to_string(), Some(error)),
    };
    let parent = format!("{}^1", selected.sha);

    match git_diff(&["diff", &parent, &selected.sha]) {
        Ok(patch) => (
            patch,
            format!("{} {}", selected.short_sha, selected.subject),
            None,
        ),
        Err(e) => (String::new(), "Commit".to_string(), Some(e)),
    }
}

fn run_uncommitted_diff() -> (String, String, Option<String>) {
    let mut patch = match git_diff(&["diff"]) {
        Ok(p) => p,
        Err(e) => return (String::new(), String::new(), Some(e)),
    };

    if let Ok(untracked) = untracked_file_patches() {
        patch.push_str(&untracked);
    }

    (patch, "Unstaged changes".to_string(), None)
}

fn merge_base(default_branch: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["merge-base", "HEAD", default_branch])
        .output()
        .map_err(|e| e.to_string())?;
    let base = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if base.is_empty() {
        return Err(format!("Could not find merge base with {default_branch}"));
    }
    Ok(base)
}

fn untracked_file_patches() -> Result<String, String> {
    let output = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()
        .map_err(|e| e.to_string())?;

    let files = String::from_utf8_lossy(&output.stdout);
    let mut patches = String::new();

    for file in files.lines().filter(|l| !l.is_empty()) {
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let line_count = content.lines().count();
        patches.push_str(&format!("diff --git a/{file} b/{file}\n"));
        patches.push_str("new file mode 100644\n");
        patches.push_str("--- /dev/null\n");
        patches.push_str(&format!("+++ b/{file}\n"));
        patches.push_str(&format!("@@ -0,0 +1,{line_count} @@\n"));
        for line in content.lines() {
            patches.push_str(&format!("+{line}\n"));
        }
    }

    Ok(patches)
}

pub fn get_all_file_contents(
    raw_patch: &str,
    source: &DiffSource,
    default_branch: &str,
) -> Vec<FileContents> {
    let (old_ref, new_ref) = match source {
        DiffSource::Branch => (merge_base(default_branch).ok(), None),
        DiffSource::Commit(commit) => match resolve_branch_commit(default_branch, commit) {
            Ok(commit) => (Some(format!("{}^1", commit.sha)), Some(commit.sha)),
            Err(_) => (None, None),
        },
        DiffSource::Uncommitted => (Some("HEAD".to_string()), None),
    };

    struct PatchPath {
        old_path: String,
        new_path: String,
    }

    let paths: Vec<PatchPath> = raw_patch
        .lines()
        .filter(|l| l.starts_with("diff --git "))
        .filter_map(|l| {
            let rest = l.strip_prefix("diff --git ")?;
            let (old_path, new_path) = rest.split_once(" b/")?;
            Some(PatchPath {
                old_path: old_path.strip_prefix("a/").unwrap_or(old_path).to_string(),
                new_path: new_path.to_string(),
            })
        })
        .collect();

    paths
        .into_iter()
        .map(|path| {
            let old_content = old_ref
                .as_ref()
                .and_then(|r| git_show_file(r, &path.old_path));
            let new_content = match &new_ref {
                Some(r) => git_show_file(r, &path.new_path),
                None => std::fs::read_to_string(&path.new_path).ok(),
            };

            FileContents {
                path: path.new_path,
                old_content,
                new_content,
            }
        })
        .collect()
}

fn git_show_file(ref_name: &str, path: &str) -> Option<String> {
    Command::new("git")
        .args(["show", &format!("{ref_name}:{path}")])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};

    use serial_test::serial;
    use tempfile::tempdir;

    use super::*;

    static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn run_git(repo: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn write_and_commit(repo: &Path, file: &str, content: &str, message: &str) -> String {
        std::fs::write(repo.join(file), content).unwrap();
        run_git(repo, &["add", file]);
        run_git(repo, &["commit", "-m", message]);
        run_git(repo, &["rev-parse", "HEAD"])
    }

    fn setup_repo() -> tempfile::TempDir {
        let repo = tempdir().unwrap();
        run_git(repo.path(), &["init", "-b", "main"]);
        run_git(repo.path(), &["config", "user.email", "orca@example.test"]);
        run_git(repo.path(), &["config", "user.name", "Orca Test"]);
        write_and_commit(repo.path(), "base.txt", "base\n", "base");
        run_git(repo.path(), &["checkout", "-b", "feature"]);
        repo
    }

    #[test]
    #[serial]
    fn commit_options_are_branch_only_newest_first() {
        let _guard = CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let repo = setup_repo();
        std::env::set_current_dir(repo.path()).unwrap();

        let first = write_and_commit(repo.path(), "one.txt", "one\n", "first feature commit");
        let second = write_and_commit(repo.path(), "two.txt", "two\n", "second feature commit");

        let commits = list_branch_commits("main").unwrap();

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].sha, second);
        assert_eq!(commits[0].subject, "second feature commit");
        assert_eq!(commits[1].sha, first);
        assert_eq!(commits[1].subject, "first feature commit");
    }

    #[test]
    #[serial]
    fn default_diff_source_uses_newest_branch_commit_when_available() {
        let _guard = CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let repo = setup_repo();
        std::env::set_current_dir(repo.path()).unwrap();

        write_and_commit(repo.path(), "one.txt", "one\n", "first feature commit");
        let newest = write_and_commit(repo.path(), "two.txt", "two\n", "second feature commit");

        let source = default_diff_source("main");

        match source {
            DiffSource::Commit(sha) => assert_eq!(sha, newest),
            DiffSource::Uncommitted | DiffSource::Branch => {
                panic!("expected newest branch commit as default source")
            }
        }
    }

    #[test]
    #[serial]
    fn commit_options_include_body_description_when_present() {
        let _guard = CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let repo = setup_repo();
        std::env::set_current_dir(repo.path()).unwrap();

        let described = write_and_commit(
            repo.path(),
            "described.txt",
            "described\n",
            "described commit\n\nExplain why this change exists.\n\nAdd operational context.",
        );
        write_and_commit(repo.path(), "plain.txt", "plain\n", "plain commit");

        let commits = list_branch_commits("main").unwrap();

        let described_commit = commits
            .iter()
            .find(|commit| commit.sha == described)
            .expect("described commit should be selectable");
        assert_eq!(
            described_commit.description.as_deref(),
            Some("Explain why this change exists.\n\nAdd operational context."),
        );

        let plain_commit = commits
            .iter()
            .find(|commit| commit.subject == "plain commit")
            .expect("plain commit should be selectable");
        assert_eq!(plain_commit.description, None);
    }

    #[test]
    #[serial]
    fn commit_diff_returns_only_selected_commit_patch_and_contents() {
        let _guard = CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let repo = setup_repo();
        std::env::set_current_dir(repo.path()).unwrap();

        write_and_commit(repo.path(), "one.txt", "one\n", "first feature commit");
        let selected = write_and_commit(repo.path(), "two.txt", "two\n", "second feature commit");

        let source = DiffSource::Commit(selected);
        let (patch, git_ref, error) = run_diff(&source, "main");
        let contents = get_all_file_contents(&patch, &source, "main");

        assert_eq!(error, None);
        assert!(git_ref.contains("second feature commit"));
        assert!(patch.contains("diff --git a/two.txt b/two.txt"));
        assert!(!patch.contains("diff --git a/one.txt b/one.txt"));
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].path, "two.txt");
        assert_eq!(contents[0].old_content, None);
        assert_eq!(contents[0].new_content.as_deref(), Some("two\n"));
    }

    #[test]
    #[serial]
    fn commit_diff_rejects_non_selectable_commit() {
        let _guard = CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let repo = setup_repo();
        std::env::set_current_dir(repo.path()).unwrap();

        let base = run_git(repo.path(), &["rev-parse", "main"]);
        let (_patch, _git_ref, error) = run_diff(&DiffSource::Commit(base.clone()), "main");

        assert_eq!(
            error.as_deref(),
            Some(format!("Commit {base} is not selectable on this branch").as_str())
        );
    }

    #[test]
    #[serial]
    fn branch_and_uncommitted_diffs_keep_existing_behavior() {
        let _guard = CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let repo = setup_repo();
        std::env::set_current_dir(repo.path()).unwrap();

        write_and_commit(
            repo.path(),
            "committed.txt",
            "committed\n",
            "feature commit",
        );
        std::fs::write(repo.path().join("tracked.txt"), "tracked\n").unwrap();
        run_git(repo.path(), &["add", "tracked.txt"]);
        run_git(repo.path(), &["commit", "-m", "tracked commit"]);
        std::fs::write(repo.path().join("tracked.txt"), "modified\n").unwrap();
        std::fs::write(repo.path().join("untracked.txt"), "untracked\n").unwrap();

        let (branch_patch, branch_ref, branch_error) = run_diff(&DiffSource::Branch, "main");
        let (worktree_patch, worktree_ref, worktree_error) =
            run_diff(&DiffSource::Uncommitted, "main");

        assert_eq!(branch_error, None);
        assert_eq!(branch_ref, "vs main");
        assert!(branch_patch.contains("diff --git a/committed.txt b/committed.txt"));
        assert!(branch_patch.contains("diff --git a/tracked.txt b/tracked.txt"));
        assert!(branch_patch.contains("diff --git a/untracked.txt b/untracked.txt"));

        assert_eq!(worktree_error, None);
        assert_eq!(worktree_ref, "Unstaged changes");
        assert!(!worktree_patch.contains("diff --git a/committed.txt b/committed.txt"));
        assert!(worktree_patch.contains("diff --git a/tracked.txt b/tracked.txt"));
        assert!(worktree_patch.contains("diff --git a/untracked.txt b/untracked.txt"));
    }

    #[test]
    #[serial]
    fn renamed_commit_uses_old_and_new_paths_for_contents() {
        let _guard = CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let repo = setup_repo();
        std::env::set_current_dir(repo.path()).unwrap();

        write_and_commit(repo.path(), "old.txt", "shared\nbefore\n", "add old file");
        run_git(repo.path(), &["mv", "old.txt", "new.txt"]);
        std::fs::write(repo.path().join("new.txt"), "shared\nafter\n").unwrap();
        run_git(repo.path(), &["add", "new.txt"]);
        run_git(repo.path(), &["commit", "-m", "rename file"]);
        let selected = run_git(repo.path(), &["rev-parse", "HEAD"]);

        let source = DiffSource::Commit(selected);
        let (patch, _git_ref, error) = run_diff(&source, "main");
        let contents = get_all_file_contents(&patch, &source, "main");

        assert_eq!(error, None);
        assert!(patch.contains("diff --git a/old.txt b/new.txt"));
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].path, "new.txt");
        assert_eq!(contents[0].old_content.as_deref(), Some("shared\nbefore\n"));
        assert_eq!(contents[0].new_content.as_deref(), Some("shared\nafter\n"));
    }

    #[test]
    #[serial]
    fn deleted_commit_does_not_read_recreated_worktree_file_as_new_content() {
        let _guard = CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let repo = setup_repo();
        std::env::set_current_dir(repo.path()).unwrap();

        write_and_commit(repo.path(), "gone.txt", "before delete\n", "add file");
        std::fs::remove_file(repo.path().join("gone.txt")).unwrap();
        run_git(repo.path(), &["add", "gone.txt"]);
        run_git(repo.path(), &["commit", "-m", "delete file"]);
        let deleted_commit = run_git(repo.path(), &["rev-parse", "HEAD"]);
        write_and_commit(
            repo.path(),
            "gone.txt",
            "recreated later\n",
            "recreate file",
        );

        let source = DiffSource::Commit(deleted_commit);
        let (patch, _git_ref, error) = run_diff(&source, "main");
        let contents = get_all_file_contents(&patch, &source, "main");

        assert_eq!(error, None);
        assert!(patch.contains("diff --git a/gone.txt b/gone.txt"));
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].path, "gone.txt");
        assert_eq!(contents[0].old_content.as_deref(), Some("before delete\n"));
        assert_eq!(contents[0].new_content, None);
    }
}
