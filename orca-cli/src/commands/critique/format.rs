use std::collections::BTreeMap;

use super::types::{Annotation, FeedbackPayload};

pub fn format_feedback(payload: &FeedbackPayload) -> String {
    if payload.overall_comment.is_empty() && payload.annotations.is_empty() {
        return "Code review completed — no changes requested.".to_string();
    }

    let mut parts = vec!["# Code Review Feedback\n".to_string()];

    if !payload.overall_comment.is_empty() {
        parts.push(payload.overall_comment.clone());
    }

    if !payload.annotations.is_empty() {
        let mut grouped: BTreeMap<&str, Vec<&Annotation>> = BTreeMap::new();
        for ann in &payload.annotations {
            grouped.entry(&ann.file_path).or_default().push(ann);
        }

        for (file_path, mut anns) in grouped {
            parts.push(format!("## {file_path}\n"));
            anns.sort_by_key(|a| a.line_start);
            for ann in anns {
                let line_range = if ann.line_start == ann.line_end {
                    format!("Line {}", ann.line_start)
                } else {
                    format!("Lines {}-{}", ann.line_start, ann.line_end)
                };
                parts.push(format!("### {} ({})\n{}", line_range, ann.side, ann.text));
            }
        }
    }

    parts.push("\nAddress all feedback above.".to_string());
    parts.join("\n\n")
}
