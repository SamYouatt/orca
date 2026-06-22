use std::collections::{BTreeMap, BTreeSet};

use super::types::{Annotation, FeedbackPayload};

pub fn format_feedback(payload: &FeedbackPayload) -> String {
    if payload.overall_comment.is_empty() && payload.annotations.is_empty() {
        return "Code review completed — no changes requested.".to_string();
    }

    let mut parts = vec!["# Code Review Feedback".to_string()];

    if !payload.overall_comment.is_empty() {
        parts.push(payload.overall_comment.clone());
    }

    if !payload.annotations.is_empty() {
        let mut grouped: BTreeMap<(Option<&str>, &str), Vec<&Annotation>> = BTreeMap::new();
        let mut scopes = BTreeSet::new();
        for ann in &payload.annotations {
            scopes.insert(ann.review_scope.as_deref());
            grouped
                .entry((ann.review_scope.as_deref(), &ann.file_path))
                .or_default()
                .push(ann);
        }

        for scope in scopes {
            parts.push(format!("## {}", scope.unwrap_or("Review")));
            for ((_, file_path), anns) in grouped
                .iter()
                .filter(|((group_scope, _), _)| *group_scope == scope)
            {
                let mut anns = anns.clone();
                parts.push(format!("### {file_path}"));
                anns.sort_by_key(|a| a.line_start);
                for ann in anns {
                    let line_range = if ann.line_start == ann.line_end {
                        format!("Line {}", ann.line_start)
                    } else {
                        format!("Lines {}-{}", ann.line_start, ann.line_end)
                    };
                    parts.push(format!("#### {} ({})\n{}", line_range, ann.side, ann.text));
                }
            }
        }
    }

    parts.push("Address all feedback above.".to_string());
    parts.join("\n\n")
}
