import React from "react";
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

let options: Record<string, unknown> | undefined;
let className: string | undefined;

mock.module("@pierre/diffs/react", () => ({
  FileDiff: ({
    className: fileDiffClassName,
    options: fileDiffOptions,
  }: {
    className: string;
    options: Record<string, unknown>;
  }) => {
    className = fileDiffClassName;
    options = fileDiffOptions;
    return <div />;
  },
}));

const { DiffViewer } = await import("./DiffViewer");

describe("DiffViewer", () => {
  test("makes its file header sticky above diff content", () => {
    renderToStaticMarkup(
      <DiffViewer
        filePath="src/example.ts"
        patch="diff --git a/src/example.ts b/src/example.ts\nindex 0000000..1111111 100644\n--- a/src/example.ts\n+++ b/src/example.ts\n@@ -0,0 +1 @@\n+export {};\n"
        annotations={[]}
        diffStyle="unified"
        themeType="light"
        viewed={false}
        collapsed={false}
        onToggleViewed={() => {}}
        onCollapsedChange={() => {}}
        onAddAnnotation={() => {}}
        onDeleteAnnotation={() => {}}
        onEditAnnotation={() => {}}
      />,
    );

    expect(options?.unsafeCSS).toContain("[data-diffs-header]");
    expect(options?.unsafeCSS).toContain("position: sticky");
    expect(options?.unsafeCSS).toContain("top: -1rem");
    expect(options?.unsafeCSS).toContain("z-index: 4");
    expect(options?.unsafeCSS).toContain("border-radius: 0.5rem 0.5rem 0 0");
    expect(options?.unsafeCSS).toContain("[data-diffs-header]::before");
    expect(options?.unsafeCSS).toContain("background: var(--muted)");
    expect(options?.unsafeCSS).toContain("border-radius: 0 0 0.5rem 0.5rem");
    expect(className).not.toContain("overflow-hidden");
  });
});
