use std::collections::BTreeMap;

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
        let mut grouped: BTreeMap<&str, Vec<&Annotation>> = BTreeMap::new();
        for ann in &payload.annotations {
            grouped.entry(&ann.file_path).or_default().push(ann);
        }

        for (file_path, anns) in grouped {
            let mut anns = anns.clone();
            parts.push(format!("## {file_path}"));
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

    parts.push("Address all feedback above.".to_string());
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn annotation(file_path: &str, line_start: u32, text: &str) -> Annotation {
        Annotation {
            file_path: file_path.to_string(),
            side: "additions".to_string(),
            line_start,
            line_end: line_start,
            text: text.to_string(),
            review_scope: None,
        }
    }

    #[test]
    fn groups_comments_by_file_and_sorts_by_line() {
        let payload = FeedbackPayload {
            overall_comment: String::new(),
            annotations: vec![
                annotation("zeta.ts", 9, "Later zeta"),
                annotation("alpha.ts", 8, "Later alpha"),
                annotation("alpha.ts", 2, "Earlier alpha"),
            ],
        };

        assert_eq!(
            format_feedback(&payload),
            "# Code Review Feedback\n\n## alpha.ts\n\n### Line 2 (additions)\nEarlier alpha\n\n### Line 8 (additions)\nLater alpha\n\n## zeta.ts\n\n### Line 9 (additions)\nLater zeta\n\nAddress all feedback above."
        );
    }

    #[test]
    fn omits_review_scope_headings_and_preserves_duplicate_comments() {
        let mut first = annotation("same.ts", 4, "Duplicate-looking feedback");
        first.review_scope = Some("Uncommitted: feature".to_string());
        let mut second = annotation("same.ts", 4, "Duplicate-looking feedback");
        second.review_scope = Some("Commit: add thing".to_string());
        let payload = FeedbackPayload {
            overall_comment: String::new(),
            annotations: vec![first, second],
        };

        assert_eq!(
            format_feedback(&payload),
            "# Code Review Feedback\n\n## same.ts\n\n### Line 4 (additions)\nDuplicate-looking feedback\n\n### Line 4 (additions)\nDuplicate-looking feedback\n\nAddress all feedback above."
        );
    }
}
