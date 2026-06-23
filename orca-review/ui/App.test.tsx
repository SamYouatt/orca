import React from "react";
import { afterEach, describe, expect, test } from "bun:test";
import { Window } from "happy-dom";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { renderToStaticMarkup } from "react-dom/server";
import { ReviewScopeTitle } from "./App";
import type { CommitOption, DiffData } from "./types";

const commitWithDescription: CommitOption = {
  sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  shortSha: "aaaaaaa",
  subject: "Add described behavior",
  description: "Explain why this commit exists.",
};

const plainCommit: CommitOption = {
  sha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  shortSha: "bbbbbbb",
  subject: "Plain commit",
};

const oldestCommit: CommitOption = {
  sha: "cccccccccccccccccccccccccccccccccccccccc",
  shortSha: "ccccccc",
  subject: "Oldest commit",
};

function diffData(
  selectedCommit: CommitOption = commitWithDescription,
  commitOptions: CommitOption[] = [selectedCommit],
): DiffData {
  return {
    rawPatch: "",
    gitRef: "aaaaaaa Add described behavior",
    diffType: "commit",
    currentBranch: "feature/descriptions",
    defaultBranch: "main",
    commitOptions,
    selectedCommit,
    files: [],
  };
}

let root: Root | null = null;
let container: HTMLElement | null = null;

afterEach(() => {
  if (root) {
    act(() => root?.unmount());
  }

  root = null;
  container = null;
  delete (globalThis as typeof globalThis & { window?: Window }).window;
  delete (globalThis as typeof globalThis & { document?: Document }).document;
  delete (globalThis as typeof globalThis & { HTMLElement?: typeof HTMLElement }).HTMLElement;
  delete (globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT;
});

function renderTitle(
  diff: DiffData,
  onSwitch: (diffType: "uncommitted" | "branch" | "commit", commitSha?: string) => void = () => {},
) {
  const window = new Window();
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  globalThis.window = window as unknown as Window & typeof globalThis;
  globalThis.document = window.document as unknown as Document;
  globalThis.HTMLElement = window.HTMLElement as unknown as typeof HTMLElement;
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);

  act(() => {
    root?.render(<ReviewScopeTitle diff={diff} switching={false} onSwitch={onSwitch} />);
  });

  return {
    rerender(nextDiff: DiffData) {
      act(() => {
        root?.render(<ReviewScopeTitle diff={nextDiff} switching={false} onSwitch={onSwitch} />);
      });
    },
    button(label: string) {
      return container?.querySelector<HTMLButtonElement>(`button[aria-label="${label}"]`) ?? null;
    },
    description() {
      return container?.querySelector("div[aria-hidden]") ?? null;
    },
    click(button: HTMLButtonElement | null) {
      act(() => {
        button?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      });
    },
  };
}

describe("ReviewScopeTitle", () => {
  test("renders an ellipsis toggle for selected commits with descriptions", () => {
    const markup = renderToStaticMarkup(
      <ReviewScopeTitle diff={diffData()} switching={false} onSwitch={() => {}} />,
    );

    expect(markup).toContain('aria-label="Show commit description"');
    expect(markup).toContain('aria-expanded="false"');
    expect(markup).toContain('aria-hidden="true"');
    expect(markup).toContain("grid-rows-[0fr]");
  });

  test("omits the description toggle for selected commits without descriptions", () => {
    const markup = renderToStaticMarkup(
      <ReviewScopeTitle
        diff={diffData(plainCommit)}
        switching={false}
        onSwitch={() => {}}
      />,
    );

    expect(markup).not.toContain("Show commit description");
    expect(markup).not.toContain("Hide commit description");
  });

  test("toggles the selected commit description and resets when the commit changes", () => {
    const view = renderTitle(diffData(commitWithDescription, [commitWithDescription, plainCommit]));

    expect(view.button("Show commit description")).not.toBeNull();
    expect(view.description()?.getAttribute("aria-hidden")).toBe("true");

    view.click(view.button("Show commit description"));

    expect(view.button("Hide commit description")).not.toBeNull();
    expect(view.description()?.getAttribute("aria-hidden")).toBe("false");
    expect(container?.textContent).toContain("Explain why this commit exists.");

    view.click(view.button("Hide commit description"));

    expect(view.button("Show commit description")).not.toBeNull();
    expect(view.description()?.getAttribute("aria-hidden")).toBe("true");

    view.click(view.button("Show commit description"));
    expect(view.description()?.getAttribute("aria-hidden")).toBe("false");

    view.rerender(diffData(plainCommit, [commitWithDescription, plainCommit]));
    expect(view.button("Show commit description")).toBeNull();
    expect(view.button("Hide commit description")).toBeNull();

    view.rerender(diffData(commitWithDescription, [commitWithDescription, plainCommit]));
    expect(view.button("Show commit description")).not.toBeNull();
    expect(view.description()?.getAttribute("aria-hidden")).toBe("true");
  });

  test("shows commit navigation only when a commit exists in that direction", () => {
    const newest = commitWithDescription;
    const middle = plainCommit;
    const oldest = oldestCommit;
    const commitsNewestFirst = [newest, middle, oldest];

    const middleMarkup = renderToStaticMarkup(
      <ReviewScopeTitle
        diff={diffData(middle, commitsNewestFirst)}
        switching={false}
        onSwitch={() => {}}
      />,
    );
    expect(middleMarkup).toContain("Prev");
    expect(middleMarkup).toContain("Next");

    const newestMarkup = renderToStaticMarkup(
      <ReviewScopeTitle
        diff={diffData(newest, commitsNewestFirst)}
        switching={false}
        onSwitch={() => {}}
      />,
    );
    expect(newestMarkup).toContain("Prev");
    expect(newestMarkup).not.toContain("Next");

    const oldestMarkup = renderToStaticMarkup(
      <ReviewScopeTitle
        diff={diffData(oldest, commitsNewestFirst)}
        switching={false}
        onSwitch={() => {}}
      />,
    );
    expect(oldestMarkup).not.toContain("Prev");
    expect(oldestMarkup).toContain("Next");
  });

  test("commit navigation switches to older and newer commits", () => {
    const switches: Array<["uncommitted" | "branch" | "commit", string | undefined]> = [];
    const newest = commitWithDescription;
    const middle = plainCommit;
    const oldest = oldestCommit;
    const view = renderTitle(diffData(middle, [newest, middle, oldest]), (diffType, commitSha) => {
      switches.push([diffType, commitSha]);
    });

    view.click(view.button("Prev commit"));
    view.click(view.button("Next commit"));

    expect(switches).toEqual([
      ["commit", oldest.sha],
      ["commit", newest.sha],
    ]);
  });
});
