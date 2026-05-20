use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};

use crate::git;
use crate::issue::{Issue, IssueId};

pub fn create(base_dir: &Path, repo: Option<&Path>, title: &str, body: &str) -> Result<IssueId> {
    let repo_path = resolve_repo(repo)?.display().to_string();
    let conn = open_store(base_dir)?;
    let tx = conn.unchecked_transaction()?;

    let next_id: u64 = tx.query_row(
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

    Ok(IssueId::from(next_id))
}

pub fn get(base_dir: &Path, repo: Option<&Path>, local_id: IssueId) -> Result<Issue> {
    let repo_path = resolve_repo(repo)?.display().to_string();
    let conn = open_store(base_dir)?;

    conn.query_row(
        "SELECT local_id, repo_path, title, body, status, created_at
         FROM issues
         WHERE repo_path = ?1 AND local_id = ?2",
        params![repo_path, local_id.as_u64()],
        issue_from_row,
    )
    .optional()?
    .with_context(|| format!("issue {local_id} not found"))
}

pub fn list(base_dir: &Path, repo: Option<&Path>, statuses: &[String]) -> Result<Vec<Issue>> {
    let repo_path = resolve_repo(repo)?.display().to_string();
    let conn = open_store(base_dir)?;

    let mut stmt = conn.prepare(
        "SELECT local_id, repo_path, title, body, status, created_at
         FROM issues
         WHERE repo_path = ?1
         ORDER BY local_id ASC",
    )?;

    let issues = stmt
        .query_map(params![repo_path], issue_from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .filter(|issue| statuses.is_empty() || statuses.contains(&issue.status))
        .collect();

    Ok(issues)
}

fn resolve_repo(repo: Option<&Path>) -> Result<PathBuf> {
    match repo {
        Some(path) => git::repo_root_from(path),
        None => git::repo_root().context("could not resolve git repository from current directory"),
    }
}

fn open_store(base_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(base_dir)?;
    // Opening the SQLite file here is the lazy store initialization path.
    let conn = Connection::open(base_dir.join("orca.db"))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS issues (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repo_path TEXT NOT NULL,
            local_id INTEGER NOT NULL CHECK(local_id >= 0),
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

fn issue_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Issue> {
    Ok(Issue {
        local_id: IssueId::from(row.get::<_, u64>(0)?),
        repo_path: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        status: row.get(4)?,
        created_at: row.get(5)?,
        blockers: Vec::new(),
    })
}
