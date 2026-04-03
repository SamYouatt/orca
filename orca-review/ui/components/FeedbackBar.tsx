import React, { useState } from "react";
import { Button } from "@/components/ui/button";

interface FeedbackBarProps {
  annotationCount: number;
  onSubmit: () => void;
  onCopyMarkdown: () => void;
}

export function FeedbackBar({ annotationCount, onSubmit, onCopyMarkdown }: FeedbackBarProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    onCopyMarkdown();
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

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
          <Button variant="outline" size="sm" onClick={handleCopy}>
            {copied ? "Copied!" : "Copy Markdown"}
          </Button>
          <Button size="sm" onClick={onSubmit}>
            Send Feedback
          </Button>
        </div>
      </div>
    </footer>
  );
}
