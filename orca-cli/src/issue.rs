use std::fmt;

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Issue {
    pub local_id: IssueId,
    pub repo_path: String,
    pub title: String,
    pub body: String,
    pub status: String,
    pub created_at: String,
    pub blockers: Vec<String>,
    pub blocked: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct IssueId(u64);

impl IssueId {
    pub fn parse(id: &str) -> Result<Self> {
        id.parse::<u64>()
            .map(Self)
            .with_context(|| format!("invalid issue id '{}'", id))
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for IssueId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for IssueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}", self.0)
    }
}
