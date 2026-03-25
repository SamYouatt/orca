use std::path::Path;
use std::process::Command;

use serde::Deserialize;

#[derive(Debug)]
pub struct PrInfo {
    pub number: u32,
    pub state: String,
    pub review_status: Option<String>,
    pub check_status: Option<CheckStatus>,
}

#[derive(Debug)]
pub enum CheckStatus {
    Passing,
    Failing,
    Pending,
}

#[derive(Deserialize)]
struct GhPr {
    number: u32,
    state: String,
    #[serde(rename = "reviewDecision")]
    review_decision: Option<String>,
    #[serde(rename = "statusCheckRollup")]
    status_check_rollup: Option<Vec<GhCheck>>,
}

#[derive(Deserialize)]
struct GhCheck {
    conclusion: Option<String>,
    #[allow(dead_code)]
    status: Option<String>,
}

pub fn is_available() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

pub fn pr_for_branch(repo: &Path, branch: &str) -> Result<Option<PrInfo>, String> {
    let output = Command::new("gh")
        .args([
            "pr",
            "list",
            "--head",
            branch,
            "--json",
            "number,state,reviewDecision,statusCheckRollup",
            "--limit",
            "1",
        ])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    let prs: Vec<GhPr> = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;

    let Some(pr) = prs.into_iter().next() else {
        return Ok(None);
    };

    let check_status = pr.status_check_rollup.map(|checks| {
        if checks.is_empty() {
            return CheckStatus::Pending;
        }
        if checks.iter().any(|c| {
            c.conclusion.as_deref() == Some("FAILURE") || c.conclusion.as_deref() == Some("ERROR")
        }) {
            CheckStatus::Failing
        } else if checks
            .iter()
            .all(|c| c.conclusion.as_deref() == Some("SUCCESS"))
        {
            CheckStatus::Passing
        } else {
            CheckStatus::Pending
        }
    });

    Ok(Some(PrInfo {
        number: pr.number,
        state: pr.state,
        review_status: pr.review_decision,
        check_status,
    }))
}
