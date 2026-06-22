import React from "react";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { CommitOption, DiffType } from "@/types";

interface DiffToggleProps {
  current: DiffType;
  defaultBranch: string;
  commitOptions: CommitOption[];
  selectedCommit?: CommitOption;
  switching: boolean;
  onSwitch: (diffType: DiffType, commitSha?: string) => void;
}

export function DiffToggle({
  current,
  defaultBranch,
  commitOptions,
  selectedCommit,
  switching,
  onSwitch,
}: DiffToggleProps) {
  const selectedSha = selectedCommit?.sha ?? "";
  const commitSelectTitle =
    selectedCommit
      ? selectedCommit.subject
      : "Select a commit";
  const commitOptionsOldestFirst = [...commitOptions].reverse();
  const showCommitSelect = current === "commit" && commitOptions.length > 0;

  return (
    <div className="flex items-center gap-2 min-w-0">
      <ToggleGroup
        value={[current]}
        onValueChange={(values) => {
          const next = values[0] as DiffType | undefined;
          if (!next || next === current) return;
          if (next === "commit") {
            const firstCommit = selectedCommit?.sha ?? commitOptionsOldestFirst[0]?.sha;
            if (firstCommit) onSwitch(next, firstCommit);
            return;
          }
          onSwitch(next);
        }}
        className="bg-muted rounded-lg p-0.5"
        disabled={switching}
      >
        <ToggleGroupItem value="uncommitted" size="sm" className="text-xs px-3 py-1 aria-pressed:bg-background aria-pressed:shadow-sm">
          Uncommitted
        </ToggleGroupItem>
        <ToggleGroupItem value="branch" size="sm" className="text-xs px-3 py-1 aria-pressed:bg-background aria-pressed:shadow-sm" title={`Changes vs ${defaultBranch}`}>
          Branch
        </ToggleGroupItem>
        <ToggleGroupItem
          value="commit"
          size="sm"
          className="text-xs px-3 py-1 aria-pressed:bg-background aria-pressed:shadow-sm"
          disabled={commitOptions.length === 0}
        >
          Commit
        </ToggleGroupItem>
      </ToggleGroup>
      {showCommitSelect && (
        <select
          value={selectedSha}
          disabled={switching}
          title={commitSelectTitle}
          aria-label="Select commit"
          onChange={(event) => {
            if (event.target.value) onSwitch("commit", event.target.value);
          }}
          className="h-7 max-w-80 min-w-44 rounded-lg border border-border bg-background px-2 text-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50"
        >
          {!selectedCommit && <option value="">Select commit</option>}
          {commitOptionsOldestFirst.map((commit) => (
            <option key={commit.sha} value={commit.sha}>
              {commit.subject}
            </option>
          ))}
        </select>
      )}
    </div>
  );
}
