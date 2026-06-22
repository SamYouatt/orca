import React, { useMemo } from "react";
import { MessageSquare, PanelRightClose, PanelRightOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { Annotation } from "../types";

interface CritiqueCommentsPaneProps {
  annotations: Annotation[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

interface AnnotationGroup {
  filePath: string;
  annotations: Annotation[];
}

function formatLineRange(annotation: Annotation): string {
  const label = annotation.lineStart === annotation.lineEnd
    ? `Line ${annotation.lineStart}`
    : `Lines ${annotation.lineStart}-${annotation.lineEnd}`;

  return `${label} (${annotation.side})`;
}

function createdAtTime(annotation: Annotation): number {
  if (!annotation.createdAt) return 0;
  const timestamp = Date.parse(annotation.createdAt);
  return Number.isNaN(timestamp) ? 0 : timestamp;
}

export function CritiqueCommentsPane({
  annotations,
  open,
  onOpenChange,
}: CritiqueCommentsPaneProps) {
  const groups = useMemo<AnnotationGroup[]>(() => {
    const byFile = new Map<string, Annotation[]>();

    for (const annotation of annotations) {
      const fileAnnotations = byFile.get(annotation.filePath) ?? [];
      fileAnnotations.push(annotation);
      byFile.set(annotation.filePath, fileAnnotations);
    }

    return [...byFile.entries()]
      .sort(([fileA], [fileB]) => fileA.localeCompare(fileB))
      .map(([filePath, fileAnnotations]) => ({
        filePath,
        annotations: [...fileAnnotations].sort(
          (a, b) => createdAtTime(b) - createdAtTime(a)
        ),
      }));
  }, [annotations]);

  const count = annotations.length;
  const countLabel = `${count} comment${count === 1 ? "" : "s"}`;

  return (
    <aside
      className={`shrink-0 overflow-hidden border-l bg-card transition-[width] duration-300 ease-in-out ${
        open ? "w-80" : "w-14"
      }`}
      aria-label="Critique comments"
    >
      {open ? (
        <div className="flex h-full flex-col">
          <header className="flex h-12 shrink-0 items-center justify-between gap-2 border-b px-3">
            <div className="flex min-w-0 items-center gap-2">
              <MessageSquare className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
              <div className="min-w-0">
                <div className="text-sm font-medium">Comments</div>
                <div className="text-xs text-muted-foreground">{countLabel}</div>
              </div>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              aria-label="Collapse comments pane"
              onClick={() => onOpenChange(false)}
            >
              <PanelRightClose className="size-4" aria-hidden="true" />
            </Button>
          </header>

          <div className="min-h-0 flex-1 overflow-y-auto">
            {count === 0 ? (
              <div className="flex h-full items-center justify-center px-4 text-sm text-muted-foreground transition-opacity duration-200">
                No comments yet
              </div>
            ) : (
              <div className="divide-y">
                {groups.map((group) => (
                  <section
                    key={group.filePath}
                    className="animate-in fade-in-0 slide-in-from-top-1 duration-200"
                  >
                    <div className="sticky top-0 z-10 border-b bg-card/95 px-3 py-2 backdrop-blur">
                      <div className="truncate text-xs font-medium" title={group.filePath}>
                        {group.filePath}
                      </div>
                    </div>
                    <div className="divide-y">
                      {group.annotations.map((annotation) => (
                        <article
                          key={annotation.id}
                          className="animate-in fade-in-0 slide-in-from-top-1 px-3 py-3 duration-200 transition-colors hover:bg-muted/60"
                        >
                          <div className="mb-1.5 text-xs text-muted-foreground">
                            {formatLineRange(annotation)}
                          </div>
                          <div className="whitespace-pre-wrap break-words text-sm leading-5">
                            {annotation.text}
                          </div>
                        </article>
                      ))}
                    </div>
                  </section>
                ))}
              </div>
            )}
          </div>
        </div>
      ) : (
        <button
          type="button"
          className="flex h-full w-14 flex-col items-center gap-2 px-2 py-3 text-muted-foreground transition-colors duration-200 hover:bg-muted hover:text-foreground"
          aria-label={`Open comments pane, ${countLabel}`}
          onClick={() => onOpenChange(true)}
        >
          <PanelRightOpen className="size-4 shrink-0" aria-hidden="true" />
          <MessageSquare className="mt-1 size-4 shrink-0" aria-hidden="true" />
          <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-medium text-foreground">
            {count}
          </span>
        </button>
      )}
    </aside>
  );
}
