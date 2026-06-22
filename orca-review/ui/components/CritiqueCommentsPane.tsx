import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { MessageSquare, PanelRightClose, PanelRightOpen, Pencil, Trash2 } from "lucide-react";
import {
  AlertDialog,
  AlertDialogBackdrop,
  AlertDialogClose,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogPopup,
  AlertDialogPortal,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";
import { Textarea } from "@/components/ui/textarea";
import type { Annotation } from "../types";

interface CritiqueCommentsPaneProps {
  annotations: Annotation[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDeleteAnnotation: (id: string) => void;
  onEditAnnotation: (id: string, text: string) => void;
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

function previewComment(text: string): string {
  const singleLine = text.replace(/\s+/g, " ").trim();
  return singleLine.length <= 140 ? singleLine : `${singleLine.slice(0, 137)}...`;
}

function AutoFocusTextarea(props: React.ComponentProps<typeof Textarea>) {
  const ref = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    ref.current?.focus();
  }, []);

  return <Textarea {...props} ref={ref} />;
}

export function CritiqueCommentsPane({
  annotations,
  open,
  onOpenChange,
  onDeleteAnnotation,
  onEditAnnotation,
}: CritiqueCommentsPaneProps) {
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editText, setEditText] = useState("");
  const [pendingDelete, setPendingDelete] = useState<Annotation | null>(null);

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
  const pendingDeleteLineRange = pendingDelete ? formatLineRange(pendingDelete) : "";

  const cancelEdit = useCallback(() => {
    setEditingId(null);
    setEditText("");
  }, []);

  const saveEdit = useCallback(() => {
    if (!editingId || !editText.trim()) return;

    onEditAnnotation(editingId, editText.trim());
    cancelEdit();
  }, [cancelEdit, editText, editingId, onEditAnnotation]);

  const handleEditKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        saveEdit();
      } else if (event.key === "Escape") {
        event.preventDefault();
        cancelEdit();
      }
    },
    [cancelEdit, saveEdit]
  );

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
                          {editingId === annotation.id ? (
                            <div
                              className="space-y-2"
                              onClick={(event) => event.stopPropagation()}
                              onMouseDown={(event) => event.stopPropagation()}
                            >
                              <AutoFocusTextarea
                                className="min-h-20 resize-y bg-background text-sm leading-5"
                                value={editText}
                                onChange={(event) => setEditText(event.target.value)}
                                onKeyDown={handleEditKeyDown}
                                rows={3}
                              />
                              <div className="flex justify-end gap-2">
                                <Button
                                  type="button"
                                  variant="ghost"
                                  size="sm"
                                  onClick={cancelEdit}
                                >
                                  Cancel <Kbd>Esc</Kbd>
                                </Button>
                                <Button
                                  type="button"
                                  size="sm"
                                  disabled={!editText.trim()}
                                  onClick={saveEdit}
                                >
                                  Save <Kbd className="border-primary-foreground/25 bg-primary-foreground/15 text-primary-foreground">Mod+Enter</Kbd>
                                </Button>
                              </div>
                            </div>
                          ) : (
                            <div className="group/comment flex items-start gap-2">
                              <div className="min-w-0 flex-1 whitespace-pre-wrap break-words text-sm leading-5">
                                {annotation.text}
                              </div>
                              <Button
                                type="button"
                                variant="ghost"
                                size="icon-xs"
                                className="shrink-0 opacity-0 transition-opacity group-hover/comment:opacity-100 focus-visible:opacity-100"
                                aria-label="Edit comment"
                                onClick={() => {
                                  setEditingId(annotation.id);
                                  setEditText(annotation.text);
                                }}
                              >
                                <Pencil className="size-3" aria-hidden="true" />
                              </Button>
                              <Button
                                type="button"
                                variant="ghost"
                                size="icon-xs"
                                className="shrink-0 text-destructive opacity-0 transition-opacity hover:text-destructive group-hover/comment:opacity-100 focus-visible:opacity-100"
                                aria-label={`Delete comment on ${annotation.filePath}, ${formatLineRange(annotation)}`}
                                onClick={() => setPendingDelete(annotation)}
                              >
                                <Trash2 className="size-3" aria-hidden="true" />
                              </Button>
                            </div>
                          )}
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
      <AlertDialog
        open={pendingDelete !== null}
        onOpenChange={(nextOpen) => {
          if (!nextOpen) setPendingDelete(null);
        }}
      >
        <AlertDialogPortal>
          <AlertDialogBackdrop />
          <AlertDialogPopup>
            <AlertDialogHeader>
              <AlertDialogTitle className="text-base font-semibold">
                Delete comment?
              </AlertDialogTitle>
              <AlertDialogDescription className="text-sm text-muted-foreground">
                This removes the side-pane comment and its inline annotation.
              </AlertDialogDescription>
            </AlertDialogHeader>

            {pendingDelete && (
              <div className="space-y-3 rounded-lg border bg-muted/40 p-3 text-sm">
                <div>
                  <div className="text-xs font-medium text-muted-foreground">File</div>
                  <div className="break-words font-mono text-xs">{pendingDelete.filePath}</div>
                </div>
                <div>
                  <div className="text-xs font-medium text-muted-foreground">Line range</div>
                  <div>{pendingDeleteLineRange}</div>
                </div>
                <div>
                  <div className="text-xs font-medium text-muted-foreground">Preview</div>
                  <div className="break-words">{previewComment(pendingDelete.text)}</div>
                </div>
              </div>
            )}

            <AlertDialogFooter>
              <AlertDialogClose
                className="inline-flex h-8 items-center justify-center rounded-lg px-2.5 text-sm font-medium transition-colors hover:bg-muted focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
              >
                Cancel
              </AlertDialogClose>
              <Button
                type="button"
                variant="destructive"
                onClick={() => {
                  if (!pendingDelete) return;
                  onDeleteAnnotation(pendingDelete.id);
                  setPendingDelete(null);
                }}
              >
                Delete
              </Button>
            </AlertDialogFooter>
          </AlertDialogPopup>
        </AlertDialogPortal>
      </AlertDialog>
    </aside>
  );
}
