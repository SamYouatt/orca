use std::path::Path;

use anyhow::{Context, Result, bail};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};

use crate::git;

#[derive(Debug)]
struct Issue {
    local_id: i64,
    repo_path: String,
    title: String,
    body: String,
    status: String,
    created_at: String,
}

pub fn create(base_dir: &Path, repo: Option<&Path>, title: &str, body: &str) -> Result<String> {
    let repo_path = resolve_repo(repo)?;
    let repo_path = repo_path.display().to_string();
    let conn = open_store(base_dir)?;
    let tx = conn.unchecked_transaction()?;

    let next_id: i64 = tx.query_row(
        "SELECT COALESCE(MAX(local_id) + 1, 0) FROM issues WHERE repo_path = ?1",
        params![repo_path],
        |row| row.get(0),
    )?;
    let now = Utc::now().to_rfc3339();

    tx.execute(
        "INSERT INTO issues (repo_path, local_id, title, body, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'todo', ?5, ?5)",
        params![repo_path, next_id, title, body, now],
    )?;
    tx.commit()?;

    Ok(format_issue_id(next_id))
}

pub fn show(base_dir: &Path, repo: Option<&Path>, id: &str) -> Result<String> {
    let repo_path = resolve_repo(repo)?;
    let repo_path = repo_path.display().to_string();
    let local_id = parse_issue_id(id)?;
    let conn = open_store(base_dir)?;

    let issue = conn
        .query_row(
            "SELECT local_id, repo_path, title, body, status, created_at
             FROM issues
             WHERE repo_path = ?1 AND local_id = ?2",
            params![repo_path, local_id],
            |row| {
                Ok(Issue {
                    local_id: row.get(0)?,
                    repo_path: row.get(1)?,
                    title: row.get(2)?,
                    body: row.get(3)?,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .optional()?
        .with_context(|| format!("issue {} not found", format_issue_id(local_id)))?;

    Ok(format_issue(&issue))
}

fn resolve_repo(repo: Option<&Path>) -> Result<std::path::PathBuf> {
    match repo {
        Some(path) => git::repo_root_from(path),
        None => git::repo_root().context("could not resolve git repository from current directory"),
    }
}

fn open_store(base_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(base_dir)?;
    let conn = Connection::open(base_dir.join("orca.db"))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS issues (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repo_path TEXT NOT NULL,
            local_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(repo_path, local_id)
        );
        CREATE INDEX IF NOT EXISTS idx_issues_repo ON issues(repo_path);",
    )?;
    Ok(conn)
}

fn parse_issue_id(id: &str) -> Result<i64> {
    let parsed = id
        .parse::<i64>()
        .with_context(|| format!("invalid issue id '{}'", id))?;
    if parsed < 0 {
        bail!("invalid issue id '{}'", id);
    }
    Ok(parsed)
}

fn format_issue_id(id: i64) -> String {
    format!("{id:04}")
}

fn format_issue(issue: &Issue) -> String {
    format!(
        "id: {}\ntitle: {}\nstatus: {}\nrepo: {}\ncreated: {}\n\n{}",
        format_issue_id(issue.local_id),
        issue.title,
        issue.status,
        issue.repo_path,
        issue.created_at,
        issue.body
    )
}
