import React from "react";
import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { ReviewScopeTitle } from "./App";
import type { DiffData } from "./types";

const commitWithDescription = {
  sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  shortSha: "aaaaaaa",
  subject: "Add described behavior",
  description: "Explain why this commit exists.",
};

function diffData(selectedCommit = commitWithDescription): DiffData {
  return {
    rawPatch: "",
    gitRef: "aaaaaaa Add described behavior",
    diffType: "commit",
    currentBranch: "feature/descriptions",
    defaultBranch: "main",
    commitOptions: [selectedCommit],
    selectedCommit,
    files: [],
  };
}

describe("ReviewScopeTitle", () => {
  test("renders an ellipsis toggle for selected commits with descriptions", () => {
    const markup = renderToStaticMarkup(
      <ReviewScopeTitle diff={diffData()} switching={false} onSwitch={() => {}} />,
    );

    expect(markup).toContain('aria-label="Show commit description"');
    expect(markup).toContain('aria-expanded="false"');
    expect(markup).not.toContain("Explain why this commit exists.");
  });

  test("omits the description toggle for selected commits without descriptions", () => {
    const markup = renderToStaticMarkup(
      <ReviewScopeTitle
        diff={diffData({
          sha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
          shortSha: "bbbbbbb",
          subject: "Plain commit",
        })}
        switching={false}
        onSwitch={() => {}}
      />,
    );

    expect(markup).not.toContain("Show commit description");
    expect(markup).not.toContain("Hide commit description");
  });
});
