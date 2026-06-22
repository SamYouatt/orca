import type { Annotation, AnnotationDraft, AnnotationOrigin, DiffData, FeedbackAnnotation, FeedbackPayload } from "../types";

export type AnnotationBuckets = Record<string, Annotation[]>;

function hashString(value: string): string {
  let hash = 0;
  for (let index = 0; index < value.length; index += 1) {
    hash = Math.imul(31, hash) + value.charCodeAt(index) | 0;
  }

  return (hash >>> 0).toString(36);
}

export function reviewStateKey(
  diff: Pick<DiffData, "diffType" | "selectedCommit" | "currentBranch" | "defaultBranch" | "rawPatch">,
): string {
  if (diff.diffType === "commit") {
    return `commit:${diff.selectedCommit?.sha ?? "unselected"}`;
  }

  if (diff.diffType === "branch") {
    return `branch:${diff.currentBranch}:${diff.defaultBranch}:${hashString(diff.rawPatch)}`;
  }

  return `uncommitted:${diff.currentBranch}:${hashString(diff.rawPatch)}`;
}

export function reviewScopeLabel(
  diff: Pick<DiffData, "diffType" | "selectedCommit" | "currentBranch" | "defaultBranch">,
): string {
  if (diff.diffType === "commit" && diff.selectedCommit) {
    return `Commit: ${diff.selectedCommit.subject}`;
  }

  if (diff.diffType === "branch") {
    return `Branch: ${diff.currentBranch} vs ${diff.defaultBranch}`;
  }

  return `Uncommitted: ${diff.currentBranch}`;
}

export function annotationOrigin(
  diff: Pick<DiffData, "diffType" | "selectedCommit" | "currentBranch" | "defaultBranch">,
): AnnotationOrigin {
  if (diff.diffType === "commit" && diff.selectedCommit) {
    return {
      type: "commit",
      currentBranch: diff.currentBranch,
      commit: diff.selectedCommit,
    };
  }

  if (diff.diffType === "branch") {
    return {
      type: "branch",
      currentBranch: diff.currentBranch,
      defaultBranch: diff.defaultBranch,
    };
  }

  return {
    type: "uncommitted",
    currentBranch: diff.currentBranch,
  };
}

export function createAnnotation(
  diff: Pick<DiffData, "diffType" | "selectedCommit" | "currentBranch" | "defaultBranch">,
  annotation: AnnotationDraft,
  id: string,
  createdAt = new Date().toISOString(),
): Annotation {
  return {
    ...annotation,
    id,
    createdAt,
    origin: annotationOrigin(diff),
    reviewScope: annotation.reviewScope ?? reviewScopeLabel(diff),
  };
}

export function rememberAnnotations(
  buckets: AnnotationBuckets,
  diff: Pick<DiffData, "diffType" | "selectedCommit" | "currentBranch" | "defaultBranch" | "rawPatch">,
  annotations: Annotation[],
): AnnotationBuckets {
  return {
    ...buckets,
    [reviewStateKey(diff)]: [...annotations],
  };
}

export function annotationsForDiff(
  buckets: AnnotationBuckets,
  diff: Pick<DiffData, "diffType" | "selectedCommit" | "currentBranch" | "defaultBranch" | "rawPatch">,
): Annotation[] {
  return [...(buckets[reviewStateKey(diff)] ?? [])];
}

export function annotationsForFeedback(
  buckets: AnnotationBuckets,
): Annotation[] {
  return Object.values(buckets).flatMap((bucket) => bucket);
}

export function editAnnotationText(annotations: Annotation[], id: string, text: string): Annotation[] {
  return annotations.map((annotation) => annotation.id === id ? { ...annotation, text } : annotation);
}

export function deleteAnnotation(annotations: Annotation[], id: string): Annotation[] {
  return annotations.filter((annotation) => annotation.id !== id);
}

export function sanitizeAnnotationForFeedback(annotation: Annotation): FeedbackAnnotation {
  const { id: _id, createdAt: _createdAt, origin: _origin, ...feedbackAnnotation } = annotation;
  return feedbackAnnotation;
}

export function serializeFeedbackPayload(
  overallComment: string,
  annotations: Annotation[],
): FeedbackPayload {
  return {
    overallComment,
    annotations: annotations.map(sanitizeAnnotationForFeedback),
  };
}

export function formatFeedbackMarkdown(annotations: Annotation[]): string {
  if (annotations.length === 0) return "Code review completed — no changes requested.";

  const parts: string[] = ["# Code Review Feedback"];
  const grouped = new Map<string, Annotation[]>();
  for (const annotation of annotations) {
    const existing = grouped.get(annotation.filePath) ?? [];
    existing.push(annotation);
    grouped.set(annotation.filePath, existing);
  }

  for (const [filePath, fileAnnotations] of [...grouped.entries()].sort(
    ([left], [right]) => left.localeCompare(right),
  )) {
    parts.push(`## ${filePath}`);
    const sorted = [...fileAnnotations].sort((left, right) => left.lineStart - right.lineStart);
    for (const annotation of sorted) {
      const range =
        annotation.lineStart === annotation.lineEnd
          ? `Line ${annotation.lineStart}`
          : `Lines ${annotation.lineStart}-${annotation.lineEnd}`;
      parts.push(`### ${range} (${annotation.side})\n${annotation.text}`);
    }
  }

  parts.push("Address all feedback above.");
  return parts.join("\n\n");
}
