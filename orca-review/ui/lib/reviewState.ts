import type { Annotation, DiffData } from "../types";

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
  currentDiff: Pick<DiffData, "diffType" | "selectedCommit" | "currentBranch" | "defaultBranch" | "rawPatch">,
): Annotation[] {
  const currentKey = reviewStateKey(currentDiff);
  return Object.entries(buckets).flatMap(([key, bucket]) => {
    if (key.startsWith("commit:") || key === currentKey) {
      return bucket;
    }

    return [];
  });
}
