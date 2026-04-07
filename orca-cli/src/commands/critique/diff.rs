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
    let mut full_args = args.to_vec();
    // Force standard a/ b/ prefixes regardless of diff.mnemonicPrefix config
    full_args.push("--src-prefix=a/");
    full_args.push("--dst-prefix=b/");
    Command::new("git")
        .args(&full_args)
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
                    Ok(mut patch) => {
                        if let Ok(untracked) = untracked_file_patches() {
                            patch.push_str(&untracked);
                        }
                        (patch, format!("vs {default_branch}"), None)
                    }
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

    let mut patch = match git_diff(&["diff"]) {
        Ok(p) => p,
        Err(e) => return (String::new(), String::new(), Some(e)),
    };

    // Include untracked files as new file diffs
    if let Ok(untracked) = untracked_file_patches() {
        patch.push_str(&untracked);
    }

    (patch, "Unstaged changes".to_string(), None)
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
            Err(_) => continue, // skip binary/unreadable files
        };

        let line_count = content.lines().count();
        patches.push_str(&format!("diff --git a/{file} b/{file}\n"));
        patches.push_str("new file mode 100644\n");
        patches.push_str(&format!("--- /dev/null\n"));
        patches.push_str(&format!("+++ b/{file}\n"));
        patches.push_str(&format!("@@ -0,0 +1,{line_count} @@\n"));
        for line in content.lines() {
            patches.push_str(&format!("+{line}\n"));
        }
    }

    Ok(patches)
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
            l.strip_prefix("diff --git ")
                .and_then(|rest| rest.split(" b/").nth(1))
                .map(|s| s.to_string())
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
