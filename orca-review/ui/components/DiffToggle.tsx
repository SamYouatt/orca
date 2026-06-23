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
  const commitOptionsOldestFirst = [...commitOptions].reverse();

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
    </div>
  );
}
