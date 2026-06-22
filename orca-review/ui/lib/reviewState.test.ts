import { describe, expect, test } from "bun:test";
import {
  annotationsForFeedback,
  annotationsForDiff,
  rememberAnnotations,
  reviewStateKey,
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
};

const commentOnB: Annotation = {
  id: "b",
  filePath: "b.ts",
  side: "deletions",
  lineStart: 9,
  lineEnd: 9,
  text: "Keep this with commit B",
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
    expect(annotationsForFeedback(buckets, commitB)).toEqual([
      commentOnA,
      commentOnB,
    ]);
  });

  test("does not submit stale branch annotations after the branch patch changes", () => {
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
    expect(annotationsForFeedback(buckets, branchAfter)).toEqual([commentOnB]);
  });

  test("does not reuse branch annotations when the branch changes with an identical patch", () => {
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
    expect(annotationsForFeedback(buckets, secondBranch)).toEqual([commentOnB]);
  });
});
