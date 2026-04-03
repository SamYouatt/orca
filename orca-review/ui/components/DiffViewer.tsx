import React, { useMemo, useState, useCallback, useEffect, useRef } from "react";
import { FileDiff, type DiffLineAnnotation } from "@pierre/diffs/react";
import { getSingularPatch, processFile } from "@pierre/diffs";
import { IconCheckboxFill, IconSquircleLg, IconChevronSm } from "@pierre/icons";
import type { Annotation } from "../types";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";

function AutoFocusTextarea(props: React.ComponentProps<typeof Textarea>) {
  const ref = useRef<HTMLTextAreaElement>(null);
  useEffect(() => {
    ref.current?.focus();
  }, []);
  return <Textarea {...props} ref={ref} />;
}

interface AnnotationMeta {
  annotationId: string;
  text: string;
}

interface SelectedRange {
  start: number;
  end: number;
  side: "deletions" | "additions";
}

interface DiffViewerProps {
  filePath: string;
  oldPath?: string;
  patch: string;
  oldContent?: string | null;
  newContent?: string | null;
  annotations: Annotation[];
  themeType: "dark" | "light";
  viewed: boolean;
  onToggleViewed: () => void;
  onAddAnnotation: (ann: Omit<Annotation, "id" | "filePath">) => void;
  onDeleteAnnotation: (id: string) => void;
  onEditAnnotation: (id: string, text: string) => void;
}

export function DiffViewer({
  filePath,
  oldPath,
  patch,
  oldContent,
  newContent,
  annotations,
  themeType,
  viewed,
  onToggleViewed,
  onAddAnnotation,
  onDeleteAnnotation,
  onEditAnnotation,
}: DiffViewerProps) {
  const fileDiff = useMemo(() => {
    const base = getSingularPatch(patch);
    if (oldContent != null || newContent != null) {
      const augmented = processFile(patch, {
        isGitDiff: true,
        oldFile: oldContent != null ? { name: oldPath || filePath, contents: oldContent } : undefined,
        newFile: newContent != null ? { name: filePath, contents: newContent } : undefined,
      });
      return augmented || base;
    }
    return base;
  }, [patch, filePath, oldPath, oldContent, newContent]);

  const [editingId, setEditingId] = useState<string | null>(null);
  const [editText, setEditText] = useState("");
  const [pendingSelection, setPendingSelection] = useState<SelectedRange | null>(null);
  const [pendingText, setPendingText] = useState("");

  const lineAnnotations: DiffLineAnnotation<AnnotationMeta>[] = useMemo(() => {
    return annotations.map((ann) => ({
      side: ann.side,
      lineNumber: ann.lineEnd,
      metadata: {
        annotationId: ann.id,
        text: ann.text,
      },
    }));
  }, [annotations]);

  const handleLineSelectionEnd = useCallback(
    (range: SelectedRange | null) => {
      if (!range) {
        setPendingSelection(null);
        setPendingText("");
        return;
      }
      const start = Math.min(range.start, range.end);
      const end = Math.max(range.start, range.end);
      setPendingSelection({ start, end, side: range.side });
      setPendingText("");
    },
    []
  );

  const handleSavePending = useCallback(() => {
    if (!pendingSelection || !pendingText.trim()) return;
    onAddAnnotation({
      side: pendingSelection.side,
      lineStart: pendingSelection.start,
      lineEnd: pendingSelection.end,
      text: pendingText.trim(),
    });
    setPendingSelection(null);
    setPendingText("");
  }, [pendingSelection, pendingText, onAddAnnotation]);

  const handleSaveEdit = useCallback(() => {
    if (!editingId || !editText.trim()) return;
    onEditAnnotation(editingId, editText.trim());
    setEditingId(null);
    setEditText("");
    setPendingSelection(null);
  }, [editingId, editText, onEditAnnotation]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent, save: () => void, cancel: () => void) => {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        save();
      } else if (e.key === "Escape") {
        cancel();
      }
    },
    []
  );

  const renderAnnotation = useCallback(
    (annotation: { side: string; lineNumber: number; metadata?: AnnotationMeta }) => {
      if (!annotation.metadata) return null;
      const { annotationId, text } = annotation.metadata;

      if (editingId === annotationId) {
        return (
          <div key={`edit-${annotationId}`} className="p-3 bg-background font-sans">
            <AutoFocusTextarea
              className="w-full bg-background rounded p-2 text-sm font-sans text-foreground resize-y"
              value={editText}
              onChange={(e) => setEditText(e.target.value)}
              onKeyDown={(e) =>
                handleKeyDown(
                  e,
                  handleSaveEdit,
                  () => { setEditingId(null); setEditText(""); }
                )
              }
              rows={2}
            />
            <div className="flex gap-2 mt-3 justify-end">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => { setEditingId(null); setEditText(""); setPendingSelection(null); }}
              >
                Cancel <Kbd>Esc</Kbd>
              </Button>
              <Button size="sm" onClick={handleSaveEdit}>
                Save <Kbd className="border-primary-foreground/25 bg-primary-foreground/15 text-primary-foreground">Mod+Enter</Kbd>
              </Button>
            </div>
          </div>
        );
      }

      return (
        <div className="bg-blue-50 dark:bg-blue-950/30 py-2 px-3 font-sans group">
          <div className="flex items-center justify-between rounded-lg border bg-background px-3 py-2.5">
            <div className="flex-1 text-sm whitespace-pre-wrap">{text}</div>
            <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0 ml-2">
              <Button
                variant="ghost"
                size="sm"
                className="h-6 px-2 text-xs"
                onClick={() => {
                  setPendingSelection(null);
                  setPendingText("");
                  setEditingId(annotationId);
                  setEditText(text);
                }}
              >
                Edit
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-6 px-2 text-xs text-destructive hover:text-destructive"
                onClick={() => onDeleteAnnotation(annotationId)}
              >
                Delete
              </Button>
            </div>
          </div>
        </div>
      );
    },
    [editingId, editText, handleSaveEdit, handleKeyDown, onDeleteAnnotation, annotations]
  );


  useEffect(() => {
    setPendingSelection(null);
    setPendingText("");
    setEditingId(null);
  }, [filePath]);

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (editingId) {
          setEditingId(null);
          setEditText("");
        } else if (pendingSelection) {
          setPendingSelection(null);
          setPendingText("");
        }
      }
    };
    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [pendingSelection, editingId]);

  const allAnnotations: DiffLineAnnotation<AnnotationMeta>[] = useMemo(() => {
    const result = [...lineAnnotations];
    if (pendingSelection) {
      result.push({
        side: pendingSelection.side,
        lineNumber: pendingSelection.end,
        metadata: { annotationId: "__pending__", text: "" },
      });
    }
    return result;
  }, [lineAnnotations, pendingSelection]);

  const pendingLabel = pendingSelection
    ? pendingSelection.start === pendingSelection.end
      ? `Line ${pendingSelection.start}`
      : `Lines ${pendingSelection.start}-${pendingSelection.end}`
    : null;

  const renderAnnotationWithPending = useCallback(
    (annotation: { side: string; lineNumber: number; metadata?: AnnotationMeta }) => {
      if (annotation.metadata?.annotationId === "__pending__") {
        return (
          <div key={pendingLabel} className="p-3 bg-background font-sans">
            {pendingLabel && (
              <div className="text-xs text-muted-foreground mb-1.5 font-sans">{pendingLabel}</div>
            )}
            <AutoFocusTextarea
              className="w-full bg-background rounded p-2 text-sm font-sans text-foreground resize-y"
              placeholder="Add a comment..."
              value={pendingText}
              onChange={(e) => setPendingText(e.target.value)}
              onKeyDown={(e) =>
                handleKeyDown(
                  e,
                  handleSavePending,
                  () => { setPendingSelection(null); setPendingText(""); }
                )
              }
              rows={2}
            />
            <div className="flex gap-2 mt-3 justify-end">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => { setPendingSelection(null); setPendingText(""); }}
              >
                Cancel <Kbd>Esc</Kbd>
              </Button>
              <Button size="sm" onClick={handleSavePending}>
                Comment <Kbd className="border-primary-foreground/25 bg-primary-foreground/15 text-primary-foreground">Mod+Enter</Kbd>
              </Button>
            </div>
          </div>
        );
      }
      return renderAnnotation(annotation);
    },
    [pendingText, pendingLabel, handleSavePending, handleKeyDown, renderAnnotation]
  );

  const [collapsed, setCollapsed] = useState(false);

  const handleToggleViewed = useCallback(() => {
    onToggleViewed();
    setCollapsed((c) => !c);
  }, [onToggleViewed]);

  return (
    <FileDiff
      className="overflow-hidden rounded-lg border"
      fileDiff={fileDiff}
      options={{
        themeType,
        diffStyle: "unified",
        enableLineSelection: true,
        enableHoverUtility: false,
        hunkSeparators: "line-info",
        expansionLineCount: 20,
        collapsed,
        onLineSelectionEnd: handleLineSelectionEnd,
      }}
      lineAnnotations={allAnnotations}
      selectedLines={pendingSelection ?? null}
      renderAnnotation={renderAnnotationWithPending}

      renderHeaderPrefix={() => (
        <button
          type="button"
          onClick={() => setCollapsed((c) => !c)}
          aria-label={collapsed ? "Expand file" : "Collapse file"}
          className="inline-flex size-6 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          style={{ marginLeft: -5 }}
        >
          <IconChevronSm
            className={`transition-transform ${collapsed ? "-rotate-90" : ""}`}
          />
        </button>
      )}
      renderHeaderMetadata={() => (
        <button
          type="button"
          onClick={handleToggleViewed}
          aria-pressed={viewed}
          className={`flex items-center gap-1.5 rounded-md border py-1 pr-2 pl-1 text-xs transition-colors ${
            viewed
              ? "border-blue-400/50 bg-blue-500/20 text-blue-700 dark:text-blue-100"
              : "border-muted-foreground/20 bg-transparent text-muted-foreground hover:border-muted-foreground/35 hover:text-foreground"
          }`}
        >
          {viewed ? (
            <IconCheckboxFill className="text-blue-400" />
          ) : (
            <IconSquircleLg className="opacity-50" />
          )}
          Viewed
        </button>
      )}
    />
  );
}
