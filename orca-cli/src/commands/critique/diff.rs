use std::process::Command;

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

fn git_diff(args: &[&str]) -> Result<String, String> {
    Command::new("git")
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .map_err(|e| e.to_string())
}

pub fn run_diff(diff_type: &str, default_branch: &str) -> (String, String, Option<String>) {
    if diff_type == "branch" {
        let merge_base = Command::new("git")
            .args(["merge-base", "HEAD", default_branch])
            .output();

        return match merge_base {
            Ok(output) => {
                let base = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if base.is_empty() {
                    return (
                        String::new(),
                        format!("vs {default_branch}"),
                        Some(format!("Could not find merge base with {default_branch}")),
                    );
                }
                match git_diff(&["diff", &base.to_string()]) {
                    Ok(patch) => (patch, format!("vs {default_branch}"), None),
                    Err(e) => (String::new(), format!("vs {default_branch}"), Some(e)),
                }
            }
            Err(e) => (
                String::new(),
                format!("vs {default_branch}"),
                Some(e.to_string()),
            ),
        };
    }

    match git_diff(&["diff"]) {
        Ok(patch) => (patch, "Unstaged changes".to_string(), None),
        Err(e) => (String::new(), String::new(), Some(e)),
    }
}

use super::types::FileContents;

pub fn get_all_file_contents(
    raw_patch: &str,
    diff_type: &str,
    default_branch: &str,
) -> Vec<FileContents> {
    let old_ref = if diff_type == "branch" {
        Command::new("git")
            .args(["merge-base", "HEAD", default_branch])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
    } else {
        Some("HEAD".to_string())
    };

    let paths: Vec<String> = raw_patch
        .lines()
        .filter(|l| l.starts_with("diff --git "))
        .filter_map(|l| {
            // Handle arbitrary single-char prefixes (a/b, i/w, etc.)
            l.strip_prefix("diff --git ").and_then(|rest| {
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    // Strip the single-char prefix (e.g. "b/path" -> "path")
                    parts[1].get(2..).map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .collect();

    paths
        .into_iter()
        .map(|path| {
            let old_content = old_ref.as_ref().and_then(|r| {
                Command::new("git")
                    .args(["show", &format!("{r}:{path}")])
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            });

            let new_content = std::fs::read_to_string(&path).ok();

            FileContents {
                path,
                old_content,
                new_content,
            }
        })
        .collect()
}
