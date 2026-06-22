import { describe, expect, test } from "bun:test";
import {
  createAnnotation,
  deleteAnnotation,
  editAnnotationText,
  annotationsForFeedback,
  annotationsForDiff,
  formatFeedbackMarkdown,
  rememberAnnotations,
  reviewStateKey,
  serializeFeedbackPayload,
  type AnnotationBuckets,
} from "./reviewState";
import type { Annotation, DiffData } from "../types";

function diffData(
  diffType: DiffData["diffType"],
  selectedCommit?: DiffData["selectedCommit"],
): DiffData {
  return {
    rawPatch: "",
    gitRef: "",
    diffType,
    currentBranch: "feature/review-branch",
    defaultBranch: "main",
    commitOptions: [],
    selectedCommit,
    files: [],
  };
}

const commentOnA: Annotation = {
  id: "a",
  filePath: "a.ts",
  side: "additions",
  lineStart: 4,
  lineEnd: 4,
  text: "Keep this with commit A",
  createdAt: "2026-06-22T10:00:00.000Z",
  origin: {
    type: "commit",
    currentBranch: "feature/review-branch",
    commit: {
      sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      shortSha: "aaaaaaa",
      subject: "add alpha",
    },
  },
};

const commentOnB: Annotation = {
  id: "b",
  filePath: "b.ts",
  side: "deletions",
  lineStart: 9,
  lineEnd: 9,
  text: "Keep this with commit B",
  createdAt: "2026-06-22T10:01:00.000Z",
  origin: {
    type: "commit",
    currentBranch: "feature/review-branch",
    commit: {
      sha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      shortSha: "bbbbbbb",
      subject: "add beta",
    },
  },
};

describe("review state buckets", () => {
  test("restores annotations for the selected commit after switching away and back", () => {
    const commitA = diffData("commit", {
      sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      shortSha: "aaaaaaa",
      subject: "add alpha",
    });
    const commitB = diffData("commit", {
      sha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      shortSha: "bbbbbbb",
      subject: "add beta",
    });

    let buckets: AnnotationBuckets = {};
    buckets = rememberAnnotations(buckets, commitA, [commentOnA]);
    buckets = rememberAnnotations(buckets, commitB, [commentOnB]);

    expect(reviewStateKey(commitA)).toBe("commit:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    expect(annotationsForDiff(buckets, commitA)).toEqual([commentOnA]);
    expect(annotationsForDiff(buckets, commitB)).toEqual([commentOnB]);
    expect(annotationsForFeedback(buckets)).toEqual([
      commentOnA,
      commentOnB,
    ]);
  });

  test("includes uncommitted branch and commit annotations in the remembered comment set", () => {
    const uncommitted = diffData("uncommitted");
    const branch = {
      ...diffData("branch"),
      rawPatch: "diff --git a/branch.ts b/branch.ts\n",
    };
    const commit = diffData("commit", {
      sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      shortSha: "aaaaaaa",
      subject: "add alpha",
    });
    const uncommittedComment = createAnnotation(
      uncommitted,
      {
        filePath: "uncommitted.ts",
        side: "additions",
        lineStart: 1,
        lineEnd: 1,
        text: "Keep uncommitted",
      },
      "uncommitted-comment",
      "2026-06-22T09:00:00.000Z",
    );
    const branchComment = createAnnotation(
      branch,
      {
        filePath: "branch.ts",
        side: "additions",
        lineStart: 2,
        lineEnd: 2,
        text: "Keep branch",
      },
      "branch-comment",
      "2026-06-22T09:01:00.000Z",
    );

    let buckets: AnnotationBuckets = {};
    buckets = rememberAnnotations(buckets, uncommitted, [uncommittedComment]);
    buckets = rememberAnnotations(buckets, branch, [branchComment]);
    buckets = rememberAnnotations(buckets, commit, [commentOnA]);

    expect(annotationsForFeedback(buckets)).toEqual([
      uncommittedComment,
      branchComment,
      commentOnA,
    ]);
  });

  test("retains branch annotations after the branch patch changes", () => {
    const branchBefore = {
      ...diffData("branch"),
      rawPatch: "diff --git a/before.ts b/before.ts\n",
    };
    const branchAfter = {
      ...diffData("branch"),
      rawPatch: "diff --git a/after.ts b/after.ts\n",
    };

    let buckets: AnnotationBuckets = {};
    buckets = rememberAnnotations(buckets, branchBefore, [commentOnA]);
    buckets = rememberAnnotations(buckets, branchAfter, [commentOnB]);

    expect(annotationsForDiff(buckets, branchAfter)).toEqual([commentOnB]);
    expect(annotationsForFeedback(buckets)).toEqual([commentOnA, commentOnB]);
  });

  test("retains branch annotations when the branch changes with an identical patch", () => {
    const patch = "diff --git a/same.ts b/same.ts\n";
    const firstBranch = {
      ...diffData("branch"),
      currentBranch: "feature/first",
      rawPatch: patch,
    };
    const secondBranch = {
      ...diffData("branch"),
      currentBranch: "feature/second",
      rawPatch: patch,
    };

    let buckets: AnnotationBuckets = {};
    buckets = rememberAnnotations(buckets, firstBranch, [commentOnA]);
    buckets = rememberAnnotations(buckets, secondBranch, [commentOnB]);

    expect(annotationsForDiff(buckets, secondBranch)).toEqual([commentOnB]);
    expect(annotationsForFeedback(buckets)).toEqual([commentOnA, commentOnB]);
  });

  test("stores creation time and structured origin metadata for local annotations", () => {
    const commit = diffData("commit", {
      sha: "cccccccccccccccccccccccccccccccccccccccc",
      shortSha: "ccccccc",
      subject: "add gamma",
    });

    const annotation = createAnnotation(
      commit,
      {
        filePath: "gamma.ts",
        side: "additions",
        lineStart: 11,
        lineEnd: 12,
        text: "Explain this",
      },
      "created-comment",
      "2026-06-22T11:12:13.000Z",
    );

    expect(annotation.createdAt).toBe("2026-06-22T11:12:13.000Z");
    expect(annotation.origin).toEqual({
      type: "commit",
      currentBranch: "feature/review-branch",
      commit: {
        sha: "cccccccccccccccccccccccccccccccccccccccc",
        shortSha: "ccccccc",
        subject: "add gamma",
      },
    });
    expect(annotation.reviewScope).toBe("Commit: add gamma");
  });

  test("editing comment text preserves original creation time and origin", () => {
    const edited = editAnnotationText([commentOnA], "a", "Updated text");

    expect(edited).toEqual([
      {
        ...commentOnA,
        text: "Updated text",
      },
    ]);
    expect(edited[0].createdAt).toBe(commentOnA.createdAt);
    expect(edited[0].origin).toEqual(commentOnA.origin);
  });

  test("deletes annotations from the selected view without affecting the rest of the bucket", () => {
    expect(deleteAnnotation([commentOnA, commentOnB], "a")).toEqual([commentOnB]);
  });

  test("serializes feedback without UI-only annotation fields", () => {
    expect(serializeFeedbackPayload("Ship it", [commentOnA])).toEqual({
      overallComment: "Ship it",
      annotations: [
        {
          filePath: "a.ts",
          side: "additions",
          lineStart: 4,
          lineEnd: 4,
          text: "Keep this with commit A",
        },
      ],
    });
  });

  test("formats feedback markdown by file and line without scope headings", () => {
    const uncommittedComment: Annotation = {
      ...commentOnA,
      id: "uncommitted",
      filePath: "zeta.ts",
      lineStart: 9,
      lineEnd: 9,
      text: "Later zeta",
      reviewScope: "Uncommitted: feature/review-branch",
    };
    const branchComment: Annotation = {
      ...commentOnA,
      id: "branch",
      filePath: "alpha.ts",
      lineStart: 8,
      lineEnd: 8,
      text: "Later alpha",
      reviewScope: "Branch: feature/review-branch vs main",
    };
    const commitComment: Annotation = {
      ...commentOnA,
      id: "commit",
      filePath: "alpha.ts",
      lineStart: 2,
      lineEnd: 2,
      text: "Earlier alpha",
      reviewScope: "Commit: add alpha",
    };

    expect(formatFeedbackMarkdown([
      uncommittedComment,
      branchComment,
      commitComment,
    ])).toBe(
      "# Code Review Feedback\n\n## alpha.ts\n\n### Line 2 (additions)\nEarlier alpha\n\n### Line 8 (additions)\nLater alpha\n\n## zeta.ts\n\n### Line 9 (additions)\nLater zeta\n\nAddress all feedback above.",
    );
  });

  test("preserves duplicate-looking feedback comments in markdown", () => {
    const duplicateA: Annotation = {
      ...commentOnA,
      id: "duplicate-a",
      reviewScope: "Uncommitted: feature/review-branch",
    };
    const duplicateB: Annotation = {
      ...commentOnA,
      id: "duplicate-b",
      reviewScope: "Commit: add alpha",
    };

    expect(formatFeedbackMarkdown([duplicateA, duplicateB])).toBe(
      "# Code Review Feedback\n\n## a.ts\n\n### Line 4 (additions)\nKeep this with commit A\n\n### Line 4 (additions)\nKeep this with commit A\n\nAddress all feedback above.",
    );
  });
});
