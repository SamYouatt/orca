import React from "react";
import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { CritiqueCommentsPane } from "./CritiqueCommentsPane";
import type { Annotation } from "../types";

const annotation: Annotation = {
  id: "comment-1",
  filePath: "src/app.ts",
  side: "additions",
  lineStart: 12,
  lineEnd: 12,
  text: "Extract this before it grows.",
  createdAt: "2026-06-22T12:00:00.000Z",
};

function renderPane() {
  return renderToStaticMarkup(
    <CritiqueCommentsPane
      annotations={[annotation]}
      open
      unavailableAnnotationIds={new Set()}
      onOpenChange={() => {}}
      onDeleteAnnotation={() => {}}
      onEditAnnotation={() => {}}
      onJumpToAnnotation={() => {}}
    />,
  );
}

describe("CritiqueCommentsPane", () => {
  test("uses the count as the open pane header label", () => {
    const markup = renderPane();

    expect(markup).toContain("1 comment");
    expect(markup).not.toContain(">Comments<");
  });

  test("renders jump target as a button without making the row interactive", () => {
    const markup = renderPane();

    expect(markup).not.toContain('role="button"');
    expect(markup).toContain(
      'aria-label="Jump to comment on src/app.ts, Line 12 (additions)"',
    );
  });

  test("renders visible sibling actions for edit and delete", () => {
    const markup = renderPane();

    expect(markup).toContain('aria-label="Edit comment"');
    expect(markup).toContain(
      'aria-label="Delete comment on src/app.ts, Line 12 (additions)"',
    );
    expect(markup).not.toContain("opacity-0");
  });
});
