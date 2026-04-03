import React from "react";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

interface DiffToggleProps {
  current: "uncommitted" | "branch";
  defaultBranch: string;
  switching: boolean;
  onSwitch: (diffType: "uncommitted" | "branch") => void;
}

export function DiffToggle({ current, defaultBranch, switching, onSwitch }: DiffToggleProps) {
  return (
    <ToggleGroup
      value={[current]}
      onValueChange={(values) => {
        const next = values[0] as "uncommitted" | "branch" | undefined;
        if (next && next !== current) onSwitch(next);
      }}
      className="bg-muted rounded-lg p-0.5"
      disabled={switching}
    >
      <ToggleGroupItem value="uncommitted" size="sm" className="text-xs px-3 py-1 aria-pressed:bg-background aria-pressed:shadow-sm">
        Uncommited
      </ToggleGroupItem>
      <ToggleGroupItem value="branch" size="sm" className="text-xs px-3 py-1 aria-pressed:bg-background aria-pressed:shadow-sm">
        Branch
      </ToggleGroupItem>
    </ToggleGroup>
  );
}
