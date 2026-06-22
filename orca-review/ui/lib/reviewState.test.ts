import { describe, expect, test } from "bun:test";
import {
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
  });
});
