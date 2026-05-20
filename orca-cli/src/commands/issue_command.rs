use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::issue::{Issue, IssueId};
use crate::issue_store;

#[derive(Serialize)]
struct IssueSummary<'a> {
    #[serde(rename = "id")]
    #[serde(serialize_with = "serialize_issue_id")]
    local_id: IssueId,
    title: &'a str,
    status: &'a str,
    blockers: &'a [String],
}

#[derive(Serialize)]
struct IssueDetails<'a> {
    #[serde(rename = "id")]
    #[serde(serialize_with = "serialize_issue_id")]
    local_id: IssueId,
    repo_path: &'a str,
    title: &'a str,
    body: &'a str,
    status: &'a str,
    created_at: &'a str,
    blockers: &'a [String],
}

pub fn create(base_dir: &Path, repo: Option<&Path>, title: &str, body: &str) -> Result<String> {
    let id = issue_store::create(base_dir, repo, title, body)?;
    Ok(id.to_string())
}

pub fn show(base_dir: &Path, repo: Option<&Path>, id: &str) -> Result<String> {
    let issue = issue_store::get(base_dir, repo, IssueId::parse(id)?)?;
    Ok(format_issue(&issue))
}

pub fn show_json(base_dir: &Path, repo: Option<&Path>, id: &str) -> Result<String> {
    let issue = issue_store::get(base_dir, repo, IssueId::parse(id)?)?;
    serde_json::to_string_pretty(&IssueDetails::from(&issue)).context("failed to serialize issue")
}

pub fn list(
    base_dir: &Path,
    repo: Option<&Path>,
    statuses: &[String],
    json: bool,
) -> Result<String> {
    let issues = issue_store::list(base_dir, repo, statuses)?;

    if json {
        let summaries = issues.iter().map(IssueSummary::from).collect::<Vec<_>>();
        return serde_json::to_string_pretty(&summaries).context("failed to serialize issues");
    }

    Ok(issues
        .iter()
        .map(format_issue_summary)
        .collect::<Vec<_>>()
        .join("\n"))
}

fn format_issue(issue: &Issue) -> String {
    format!(
        "id: {}\ntitle: {}\nstatus: {}\nrepo: {}\ncreated: {}\n\n{}",
        issue.local_id, issue.title, issue.status, issue.repo_path, issue.created_at, issue.body
    )
}

fn format_issue_summary(issue: &Issue) -> String {
    let blockers = if issue.blockers.is_empty() {
        "-".to_string()
    } else {
        issue.blockers.join(",")
    };

    format!(
        "{}  {}  {}  blockers: {}",
        issue.local_id, issue.status, issue.title, blockers
    )
}

fn serialize_issue_id<S>(id: &IssueId, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&id.to_string())
}

impl<'a> From<&'a Issue> for IssueSummary<'a> {
    fn from(issue: &'a Issue) -> Self {
        Self {
            local_id: issue.local_id,
            title: &issue.title,
            status: &issue.status,
            blockers: &issue.blockers,
        }
    }
}

impl<'a> From<&'a Issue> for IssueDetails<'a> {
    fn from(issue: &'a Issue) -> Self {
        Self {
            local_id: issue.local_id,
            repo_path: &issue.repo_path,
            title: &issue.title,
            body: &issue.body,
            status: &issue.status,
            created_at: &issue.created_at,
            blockers: &issue.blockers,
        }
    }
}
