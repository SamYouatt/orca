import React from "react";
import { afterEach, describe, expect, test } from "bun:test";
import { Window } from "happy-dom";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { DiffToggle } from "./DiffToggle";
import type { CommitOption, DiffType } from "@/types";

const newestCommit: CommitOption = {
  sha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  shortSha: "bbbbbbb",
  subject: "Newest commit",
};

const oldestCommit: CommitOption = {
  sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  shortSha: "aaaaaaa",
  subject: "Oldest commit",
};

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

function renderToggle(onSwitch: (diffType: DiffType, commitSha?: string) => void) {
  const window = new Window();
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  globalThis.window = window as unknown as Window & typeof globalThis;
  globalThis.document = window.document as unknown as Document;
  globalThis.HTMLElement = window.HTMLElement as unknown as typeof HTMLElement;
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);

  act(() => {
    root?.render(
      <DiffToggle
        current="branch"
        defaultBranch="main"
        commitOptions={[newestCommit, oldestCommit]}
        switching={false}
        onSwitch={onSwitch}
      />,
    );
  });

  return {
    clickCommit() {
      const commitButton = Array.from(container?.querySelectorAll("button") ?? []).find(
        (button) => button.textContent === "Commit",
      );
      act(() => {
        commitButton?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      });
    },
  };
}

describe("DiffToggle", () => {
  test("selects the newest commit when switching into commit view", () => {
    const switches: Array<[DiffType, string | undefined]> = [];
    const view = renderToggle((diffType, commitSha) => {
      switches.push([diffType, commitSha]);
    });

    view.clickCommit();

    expect(switches).toEqual([["commit", newestCommit.sha]]);
  });
});
