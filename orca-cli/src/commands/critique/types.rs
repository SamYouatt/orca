use serde::{Deserialize, Serialize};

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
    pub diff_type: String,
    pub default_branch: String,
    pub files: Vec<FileContents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchRequest {
    pub diff_type: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    pub file_path: String,
    pub side: String,
    pub line_start: u32,
    pub line_end: u32,
    pub text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackPayload {
    pub overall_comment: String,
    pub annotations: Vec<Annotation>,
}
