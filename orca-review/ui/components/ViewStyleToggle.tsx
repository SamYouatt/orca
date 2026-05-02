import React from "react";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { IconDiffUnified, IconDiffSplit } from "@pierre/icons";

type DiffStyle = "unified" | "split";

interface ViewStyleToggleProps {
  current: DiffStyle;
  onChange: (style: DiffStyle) => void;
}

export function ViewStyleToggle({ current, onChange }: ViewStyleToggleProps) {
  return (
    <ToggleGroup
      value={[current]}
      onValueChange={(values) => {
        const next = values[0] as DiffStyle | undefined;
        if (next && next !== current) onChange(next);
      }}
      className="bg-muted rounded-lg p-0.5"
    >
      <ToggleGroupItem value="unified" size="sm" className="px-2 py-1 aria-pressed:bg-background aria-pressed:shadow-sm" aria-label="Unified diff">
        <IconDiffUnified className="size-4" />
      </ToggleGroupItem>
      <ToggleGroupItem value="split" size="sm" className="px-2 py-1 aria-pressed:bg-background aria-pressed:shadow-sm" aria-label="Split diff">
        <IconDiffSplit className="size-4" />
      </ToggleGroupItem>
    </ToggleGroup>
  );
}
