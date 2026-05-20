use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Transaction, params};

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

    let mut issue = conn
        .query_row(
            "SELECT local_id, repo_path, title, body, status, created_at
             FROM issues
             WHERE repo_path = ?1 AND local_id = ?2",
            params![repo_path, local_id.as_u64()],
            issue_from_row,
        )
        .optional()?
        .with_context(|| format!("issue {local_id} not found"))?;
    hydrate_dependencies(&conn, &repo_path, &mut issue)?;
    Ok(issue)
}

pub fn list(
    base_dir: &Path,
    repo: Option<&Path>,
    statuses: &[String],
    blocked_by: Option<&str>,
) -> Result<Vec<Issue>> {
    let repo_path = resolve_repo(repo)?.display().to_string();
    let blocked_by = blocked_by.map(IssueId::parse).transpose()?;
    let conn = open_store(base_dir)?;

    if let Some(blocker_id) = blocked_by {
        ensure_issue_exists(&conn, &repo_path, blocker_id)?;
    }
    let blocked_by = blocked_by.map(|id| id.to_string());

    let mut stmt = conn.prepare(
        "SELECT local_id, repo_path, title, body, status, created_at
         FROM issues
         WHERE repo_path = ?1
         ORDER BY local_id ASC",
    )?;

    let mut issues = Vec::new();
    for issue in stmt.query_map(params![repo_path], issue_from_row)? {
        let mut issue = issue?;
        hydrate_dependencies(&conn, &repo_path, &mut issue)?;
        if !statuses.is_empty() && !statuses.contains(&issue.status) {
            continue;
        }
        if let Some(blocker_id) = blocked_by.as_ref()
            && !issue.blockers.iter().any(|id| id == blocker_id)
        {
            continue;
        }
        issues.push(issue);
    }

    Ok(issues)
}

pub fn block(base_dir: &Path, repo: Option<&Path>, id: &str, blockers: &[&str]) -> Result<()> {
    mutate_blockers(base_dir, repo, id, blockers, Mutation::Add)
}

pub fn unblock(base_dir: &Path, repo: Option<&Path>, id: &str, blockers: &[&str]) -> Result<()> {
    mutate_blockers(base_dir, repo, id, blockers, Mutation::Remove)
}

fn mutate_blockers(
    base_dir: &Path,
    repo: Option<&Path>,
    id: &str,
    blockers: &[&str],
    mutation: Mutation,
) -> Result<()> {
    if blockers.is_empty() {
        bail!("at least one blocker id is required");
    }

    let repo_path = resolve_repo(repo)?.display().to_string();
    let issue_id = IssueId::parse(id)?;
    let blocker_ids = blockers
        .iter()
        .map(|blocker| IssueId::parse(blocker))
        .collect::<Result<Vec<_>>>()?;
    let conn = open_store(base_dir)?;
    let tx = conn.unchecked_transaction()?;

    ensure_issue_exists(&tx, &repo_path, issue_id)?;
    for blocker_id in &blocker_ids {
        if *blocker_id == issue_id {
            bail!("issue {issue_id} cannot block itself");
        }
        ensure_issue_exists(&tx, &repo_path, *blocker_id)?;
    }

    match mutation {
        Mutation::Add => {
            for blocker_id in blocker_ids {
                if creates_cycle(&tx, &repo_path, issue_id, blocker_id)? {
                    bail!(
                        "adding blocker {blocker_id} to issue {issue_id} would create a dependency cycle"
                    );
                }
                tx.execute(
                    "INSERT OR IGNORE INTO issue_dependencies (repo_path, issue_id, blocker_id)
                     VALUES (?1, ?2, ?3)",
                    params![repo_path, issue_id.as_u64(), blocker_id.as_u64()],
                )?;
            }
        }
        Mutation::Remove => {
            for blocker_id in blocker_ids {
                tx.execute(
                    "DELETE FROM issue_dependencies
                     WHERE repo_path = ?1 AND issue_id = ?2 AND blocker_id = ?3",
                    params![repo_path, issue_id.as_u64(), blocker_id.as_u64()],
                )?;
            }
        }
    }

    tx.commit()?;
    Ok(())
}

fn resolve_repo(repo: Option<&Path>) -> Result<PathBuf> {
    match repo {
        Some(path) => git::repo_root_from(path),
        None => git::repo_root().context("could not resolve git repository from current directory"),
    }
}

fn open_store(base_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(base_dir)?;
    let conn = Connection::open(base_dir.join("orca.db"))?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS issues (
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
        CREATE INDEX IF NOT EXISTS idx_issues_repo ON issues(repo_path);
        CREATE TABLE IF NOT EXISTS issue_dependencies (
            repo_path TEXT NOT NULL,
            issue_id INTEGER NOT NULL CHECK(issue_id >= 0),
            blocker_id INTEGER NOT NULL CHECK(blocker_id >= 0),
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY(repo_path, issue_id, blocker_id),
            FOREIGN KEY(repo_path, issue_id) REFERENCES issues(repo_path, local_id) ON DELETE CASCADE,
            FOREIGN KEY(repo_path, blocker_id) REFERENCES issues(repo_path, local_id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_issue_dependencies_blocker
            ON issue_dependencies(repo_path, blocker_id);",
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
        blocked: Vec::new(),
    })
}

fn hydrate_dependencies(conn: &Connection, repo_path: &str, issue: &mut Issue) -> Result<()> {
    issue.blockers = blocker_ids(conn, repo_path, issue.local_id)?
        .into_iter()
        .map(|id| id.to_string())
        .collect();
    issue.blocked = blocked_ids(conn, repo_path, issue.local_id)?
        .into_iter()
        .map(|id| id.to_string())
        .collect();
    Ok(())
}

fn blocker_ids(conn: &Connection, repo_path: &str, local_id: IssueId) -> Result<Vec<IssueId>> {
    let mut stmt = conn.prepare(
        "SELECT blocker_id
         FROM issue_dependencies
         WHERE repo_path = ?1 AND issue_id = ?2
         ORDER BY blocker_id ASC",
    )?;
    stmt.query_map(params![repo_path, local_id.as_u64()], |row| {
        Ok(IssueId::from(row.get::<_, u64>(0)?))
    })?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(Into::into)
}

fn blocked_ids(conn: &Connection, repo_path: &str, local_id: IssueId) -> Result<Vec<IssueId>> {
    let mut stmt = conn.prepare(
        "SELECT issue_id
         FROM issue_dependencies
         WHERE repo_path = ?1 AND blocker_id = ?2
         ORDER BY issue_id ASC",
    )?;
    stmt.query_map(params![repo_path, local_id.as_u64()], |row| {
        Ok(IssueId::from(row.get::<_, u64>(0)?))
    })?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(Into::into)
}

fn ensure_issue_exists(conn: &Connection, repo_path: &str, local_id: IssueId) -> Result<()> {
    conn.query_row(
        "SELECT 1 FROM issues WHERE repo_path = ?1 AND local_id = ?2",
        params![repo_path, local_id.as_u64()],
        |_| Ok(()),
    )
    .optional()?
    .with_context(|| format!("issue {local_id} not found"))
}

fn creates_cycle(
    tx: &Transaction<'_>,
    repo_path: &str,
    issue_id: IssueId,
    blocker_id: IssueId,
) -> Result<bool> {
    let mut seen = BTreeSet::new();
    let mut pending = VecDeque::from([blocker_id]);

    while let Some(current) = pending.pop_front() {
        if current == issue_id {
            return Ok(true);
        }
        if !seen.insert(current) {
            continue;
        }

        let mut stmt = tx.prepare(
            "SELECT blocker_id FROM issue_dependencies
             WHERE repo_path = ?1 AND issue_id = ?2",
        )?;
        for row in stmt.query_map(params![repo_path, current.as_u64()], |row| {
            Ok(IssueId::from(row.get::<_, u64>(0)?))
        })? {
            pending.push_back(row?);
        }
    }

    Ok(false)
}

#[derive(Clone, Copy)]
enum Mutation {
    Add,
    Remove,
}
