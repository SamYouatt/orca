use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiffType {
    Uncommitted,
    Branch,
    Commit,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitOption {
    pub sha: String,
    pub short_sha: String,
    pub subject: String,
}

#[derive(Clone, Debug)]
pub enum DiffSource {
    Uncommitted,
    Branch,
    Commit(String),
}

impl DiffSource {
    pub fn diff_type(&self) -> DiffType {
        match self {
            Self::Uncommitted => DiffType::Uncommitted,
            Self::Branch => DiffType::Branch,
            Self::Commit(_) => DiffType::Commit,
        }
    }

    pub fn selected_commit_sha(&self) -> Option<&str> {
        match self {
            Self::Commit(sha) => Some(sha),
            _ => None,
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileContents {
    pub path: String,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffData {
    pub raw_patch: String,
    pub git_ref: String,
    pub diff_type: DiffType,
    pub current_branch: String,
    pub default_branch: String,
    pub files: Vec<FileContents>,
    pub commit_options: Vec<CommitOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_commit: Option<CommitOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchRequest {
    pub diff_type: DiffType,
    pub commit_id: Option<String>,
}

impl TryFrom<SwitchRequest> for DiffSource {
    type Error = &'static str;

    fn try_from(request: SwitchRequest) -> Result<Self, Self::Error> {
        match request.diff_type {
            DiffType::Uncommitted => Ok(Self::Uncommitted),
            DiffType::Branch => Ok(Self::Branch),
            DiffType::Commit => request
                .commit_id
                .filter(|id| !id.trim().is_empty())
                .map(Self::Commit)
                .ok_or("commit diff requires commitId"),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    pub file_path: String,
    pub side: String,
    pub line_start: u32,
    pub line_end: u32,
    pub text: String,
    pub review_scope: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackPayload {
    pub overall_comment: String,
    pub annotations: Vec<Annotation>,
}
