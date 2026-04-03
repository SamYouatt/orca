import React from "react";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";

interface FeedbackBarProps {
  annotationCount: number;
  copied: boolean;
  onSubmit: () => void;
  onCopyMarkdown: () => void;
}

export function FeedbackBar({ annotationCount, copied, onSubmit, onCopyMarkdown }: FeedbackBarProps) {

  return (
    <footer className="border-t bg-card">
      <div className="flex items-center justify-between px-4 py-2">
        <div className="flex items-center gap-3">
          {annotationCount > 0 && (
            <span className="text-xs text-muted-foreground">
              {annotationCount} annotation{annotationCount !== 1 ? "s" : ""}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={onCopyMarkdown}>
            {copied ? "Copied!" : "Copy Markdown"} <Kbd>Mod+Shift+C</Kbd>
          </Button>
          <Button size="sm" onClick={onSubmit}>
            Send Feedback <Kbd className="border-primary-foreground/25 bg-primary-foreground/15 text-primary-foreground">Mod+Shift+Enter</Kbd>
          </Button>
        </div>
      </div>
    </footer>
  );
}
