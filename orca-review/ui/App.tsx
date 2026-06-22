import React, { useState, useEffect, useCallback, useRef, useMemo } from "react";
import "./app.css";
import { DiffToggle } from "./components/DiffToggle";
import { ViewStyleToggle } from "./components/ViewStyleToggle";
import { FileTree } from "./components/FileTree";
import { DiffViewer } from "./components/DiffViewer";
import { FeedbackBar } from "./components/FeedbackBar";
import { CritiqueCommentsPane } from "./components/CritiqueCommentsPane";
import { useTheme } from "./hooks/useTheme";
import { buildFileTree, flattenTreeFiles } from "./lib/fileTree";
import {
  annotationsForFeedback,
  annotationsForDiff,
  createAnnotation,
  deleteAnnotation,
  editAnnotationText,
  formatFeedbackMarkdown,
  rememberAnnotations,
  serializeFeedbackPayload,
  type AnnotationBuckets,
} from "./lib/reviewState";
import type { Annotation, DiffData, DiffType, ServerFileContents } from "./types";
import { GitBranch, GitCommitHorizontal } from "lucide-react";

interface DiffFile {
  path: string;
  oldPath?: string;
  patch: string;
  additions: number;
  deletions: number;
  oldContent?: string | null;
  newContent?: string | null;
}

function parentDirectories(filePath: string): string[] {
  const parts = filePath.split("/");
  parts.pop();

  return parts.map((_, index) => parts.slice(0, index + 1).join("/"));
}

function needsViewSwitch(diff: DiffData, annotation: Annotation): boolean {
  const { origin } = annotation;
  if (!origin) return false;
  if (diff.diffType !== origin.type) return true;
  if (origin.type !== "commit") return false;

  return diff.selectedCommit?.sha !== origin.commit.sha;
}

function parseDiffToFiles(rawPatch: string, serverFiles: ServerFileContents[]): DiffFile[] {
  const contentsMap = new Map(serverFiles.map((f) => [f.path, f]));
  const files: DiffFile[] = [];
  const fileChunks = rawPatch.split(/^diff --git /m).filter(Boolean);

  for (const chunk of fileChunks) {
    const lines = chunk.split("\n");
    const headerMatch = lines[0]?.match(/a\/(.+) b\/(.+)/);
    if (!headerMatch) continue;

    const oldPath = headerMatch[1];
    const newPath = headerMatch[2];

    let additions = 0;
    let deletions = 0;

    for (const line of lines) {
      if (line.startsWith("+") && !line.startsWith("+++")) additions++;
      if (line.startsWith("-") && !line.startsWith("---")) deletions++;
    }

    const contents = contentsMap.get(newPath);
    files.push({
      path: newPath,
      oldPath: oldPath !== newPath ? oldPath : undefined,
      patch: "diff --git " + chunk,
      additions,
      deletions,
      oldContent: contents?.oldContent,
      newContent: contents?.newContent,
    });
  }

  return files;
}

function ReviewScopeTitle({ diff }: { diff: DiffData }) {
  const selectedCommit = diff.diffType === "commit" ? diff.selectedCommit : undefined;

  return (
    <div className="min-w-0 max-w-full">
      <div className="flex min-w-0 items-center gap-2 px-3 py-1.5">
        <div className="flex min-w-0 items-center gap-1.5 text-sm font-medium">
          <GitBranch className="size-4 shrink-0 text-emerald-600 dark:text-emerald-400" aria-hidden="true" />
          <span className="truncate" title={diff.currentBranch}>
            {diff.currentBranch}
          </span>
        </div>
        {selectedCommit && (
          <>
            <span className="h-4 w-px shrink-0 bg-border" />
            <div className="flex min-w-0 items-center gap-1.5 text-sm text-muted-foreground">
              <GitCommitHorizontal className="size-4 shrink-0 text-amber-600 dark:text-amber-400" aria-hidden="true" />
              <span className="truncate" title={selectedCommit.subject}>
                {selectedCommit.subject}
              </span>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default function App() {
  const [diff, setDiff] = useState<DiffData | null>(null);
  const [files, setFiles] = useState<DiffFile[]>([]);
  const [activeFile, setActiveFile] = useState<string | null>(null);
  const [annotationBuckets, setAnnotationBuckets] = useState<AnnotationBuckets>({});
  const [switching, setSwitching] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [viewedFiles, setViewedFiles] = useState<Set<string>>(new Set());
  const [collapsedDirs, setCollapsedDirs] = useState<Set<string>>(new Set());
  const [collapsedFiles, setCollapsedFiles] = useState<Set<string>>(new Set());
  const [commentsPaneOpen, setCommentsPaneOpen] = useState(
    () => window.innerWidth >= 1024
  );
  const [pendingJump, setPendingJump] = useState<Annotation | null>(null);
  const [unavailableAnnotationIds, setUnavailableAnnotationIds] = useState<Set<string>>(new Set());
  const fileRefs = useRef<Map<string, HTMLDivElement>>(new Map());
  const [diffStyle, setDiffStyle] = useState<"unified" | "split">(
    () => window.innerWidth >= 1400 ? "split" : "unified"
  );
  const theme = useTheme();

  const tree = useMemo(
    () => buildFileTree(files.map((f) => ({ path: f.path, additions: f.additions, deletions: f.deletions }))),
    [files]
  );
  const orderedFiles = useMemo(() => {
    const byPath = new Map(files.map((f) => [f.path, f]));
    return flattenTreeFiles(tree)
      .map((tf) => byPath.get(tf.path))
      .filter((f): f is DiffFile => Boolean(f));
  }, [tree, files]);
  const annotations = useMemo(
    () => diff ? annotationsForDiff(annotationBuckets, diff) : [],
    [annotationBuckets, diff]
  );
  const feedbackAnnotations = useMemo(
    () => annotationsForFeedback(annotationBuckets),
    [annotationBuckets]
  );

  useEffect(() => {
    if (activeFile === null && orderedFiles.length > 0) {
      setActiveFile(orderedFiles[0].path);
    }
  }, [activeFile, orderedFiles]);

  const applyDiff = useCallback((data: DiffData) => {
    setDiff(data);
    setFiles(parseDiffToFiles(data.rawPatch, data.files || []));
    fileRefs.current.clear();
    setViewedFiles(new Set());
    setCollapsedDirs(new Set());
    setCollapsedFiles(new Set());
    setActiveFile(null);
  }, []);

  useEffect(() => {
    fetch("/api/diff")
      .then((res) => res.json())
      .then(applyDiff);
  }, [applyDiff]);

  const handleSwitch = useCallback(
    async (diffType: DiffType, commitId?: string) => {
      setSwitching(true);
      try {
        const res = await fetch("/api/diff/switch", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ diffType, commitId }),
        });
        applyDiff(await res.json());
      } finally {
        setSwitching(false);
      }
    },
    [applyDiff]
  );

  const handleAddAnnotation = useCallback(
    (ann: Omit<Annotation, "id">) => {
      if (!diff) return;

      setAnnotationBuckets((prev) => {
        const next = [
          ...annotationsForDiff(prev, diff),
          {
            ...createAnnotation(diff, ann, crypto.randomUUID()),
          },
        ];
        return rememberAnnotations(prev, diff, next);
      });
    },
    [diff]
  );

  const handleDeleteAnnotation = useCallback((id: string) => {
    if (!diff) return;

    setAnnotationBuckets((prev) => {
      const next = deleteAnnotation(annotationsForDiff(prev, diff), id);
      return rememberAnnotations(prev, diff, next);
    });
  }, [diff]);

  const handleDeleteFeedbackAnnotation = useCallback((id: string) => {
    setAnnotationBuckets((prev) => {
      const next: AnnotationBuckets = {};

      for (const [key, bucket] of Object.entries(prev)) {
        next[key] = deleteAnnotation(bucket, id);
      }

      return next;
    });
  }, []);

  const handleEditAnnotation = useCallback((id: string, text: string) => {
    setAnnotationBuckets((prev) => {
      const next: AnnotationBuckets = {};

      for (const [key, bucket] of Object.entries(prev)) {
        next[key] = editAnnotationText(bucket, id, text);
      }

      return next;
    });
  }, []);

  const handleJumpToAnnotation = useCallback(
    async (annotation: Annotation) => {
      if (!diff || !annotation.origin) {
        setUnavailableAnnotationIds((prev) => new Set(prev).add(annotation.id));
        return;
      }

      setUnavailableAnnotationIds((prev) => {
        const next = new Set(prev);
        next.delete(annotation.id);
        return next;
      });

      if (needsViewSwitch(diff, annotation)) {
        await handleSwitch(
          annotation.origin.type,
          annotation.origin.type === "commit" ? annotation.origin.commit.sha : undefined,
        );
      }

      setPendingJump(annotation);
    },
    [diff, handleSwitch]
  );

  const buildMarkdown = useCallback(() => {
    return formatFeedbackMarkdown(feedbackAnnotations);
  }, [feedbackAnnotations]);

  const handleSubmit = useCallback(async () => {
    await fetch("/api/feedback", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(serializeFeedbackPayload("", feedbackAnnotations)),
    });
    setSubmitted(true);
  }, [feedbackAnnotations]);

  const [copied, setCopied] = useState(false);
  const handleCopyMarkdown = useCallback(async () => {
    navigator.clipboard.writeText(buildMarkdown());
    setCopied(true);
    await fetch("/api/feedback", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(serializeFeedbackPayload("", feedbackAnnotations)),
    });
    setSubmitted(true);
  }, [buildMarkdown, feedbackAnnotations]);

  const scrollToFile = useCallback((filePath: string) => {
    setActiveFile(filePath);
    const el = fileRefs.current.get(filePath);
    if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
  }, []);

  useEffect(() => {
    if (!pendingJump) return;

    if (!orderedFiles.some((file) => file.path === pendingJump.filePath)) {
      setUnavailableAnnotationIds((prev) => new Set(prev).add(pendingJump.id));
      setPendingJump(null);
      return;
    }

    setActiveFile(pendingJump.filePath);
    setCollapsedDirs((prev) => {
      const next = new Set(prev);
      for (const dir of parentDirectories(pendingJump.filePath)) {
        next.delete(dir);
      }
      return next;
    });
    setCollapsedFiles((prev) => {
      const next = new Set(prev);
      next.delete(pendingJump.filePath);
      return next;
    });

    window.requestAnimationFrame(() => {
      window.requestAnimationFrame(() => {
        const fileElement = fileRefs.current.get(pendingJump.filePath);
        if (!fileElement) {
          setUnavailableAnnotationIds((prev) => new Set(prev).add(pendingJump.id));
          return;
        }

        const annotationElement = fileElement.querySelector<HTMLElement>(
          `[data-annotation-id="${pendingJump.id}"]`,
        );

        if (annotationElement) {
          annotationElement.scrollIntoView({ behavior: "smooth", block: "center" });
          setUnavailableAnnotationIds((prev) => {
            const next = new Set(prev);
            next.delete(pendingJump.id);
            return next;
          });
        } else {
          fileElement.scrollIntoView({ behavior: "smooth", block: "start" });
          setUnavailableAnnotationIds((prev) => new Set(prev).add(pendingJump.id));
        }
      });
    });

    setPendingJump(null);
  }, [orderedFiles, pendingJump]);

  useEffect(() => {
    const handleGlobalKeys = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === "Enter") {
        e.preventDefault();
        handleSubmit();
      } else if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === "c" || e.key === "C")) {
        e.preventDefault();
        handleCopyMarkdown();
      }
    };
    document.addEventListener("keydown", handleGlobalKeys);
    return () => document.removeEventListener("keydown", handleGlobalKeys);
  }, [handleSubmit, handleCopyMarkdown]);

  if (!diff) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-muted-foreground">Loading diff...</div>
      </div>
    );
  }

  if (submitted) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-center">
          <div className="text-2xl mb-2">Feedback sent</div>
          <div className="text-muted-foreground">You can close this tab.</div>
        </div>
      </div>
    );
  }

  if (diff.error && !diff.rawPatch) {
    return (
      <div className="flex flex-col items-center justify-center h-screen gap-4">
        <div className="text-destructive">{diff.error}</div>
        <DiffToggle
          current={diff.diffType}
          defaultBranch={diff.defaultBranch}
          commitOptions={diff.commitOptions || []}
          selectedCommit={diff.selectedCommit}
          switching={switching}
          onSwitch={handleSwitch}
        />
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col">
      <header className="flex items-center justify-between gap-3 px-4 py-2 border-b bg-card">
        <div className="min-w-0 flex-1">
          <ReviewScopeTitle diff={diff} />
        </div>
        <div className="flex min-w-0 items-center justify-end gap-2">
          <ViewStyleToggle current={diffStyle} onChange={setDiffStyle} />
          <DiffToggle
            current={diff.diffType}
            defaultBranch={diff.defaultBranch}
            commitOptions={diff.commitOptions || []}
            selectedCommit={diff.selectedCommit}
            switching={switching}
            onSwitch={handleSwitch}
          />
        </div>
      </header>

      <div className="relative flex flex-1 overflow-hidden">
        <aside className="hidden w-64 shrink-0 overflow-y-auto border-r bg-card lg:block">
          <FileTree
            tree={tree}
            activeFile={activeFile}
            annotations={annotations}
            collapsed={collapsedDirs}
            onToggleDir={(path) =>
              setCollapsedDirs((prev) => {
                const next = new Set(prev);
                if (next.has(path)) next.delete(path);
                else next.add(path);
                return next;
              })
            }
            onSelectFile={scrollToFile}
          />
        </aside>

        <main className="min-w-0 flex-1 overflow-y-auto p-4 bg-muted">
          {orderedFiles.length === 0 ? (
            <div className="text-muted-foreground text-center mt-20">
              No changes to review.
            </div>
          ) : (
            <div className="flex flex-col gap-4">
              {orderedFiles.map((file) => (
                <div
                  key={file.path}
                  ref={(el) => {
                    if (el) fileRefs.current.set(file.path, el);
                  }}
                >
                  <DiffViewer
                    filePath={file.path}
                    oldPath={file.oldPath}
                    patch={file.patch}
                    oldContent={file.oldContent}
                    newContent={file.newContent}
                    annotations={annotations.filter(
                      (a) => a.filePath === file.path
                    )}
                    diffStyle={diffStyle}
                    themeType={theme}
                    viewed={viewedFiles.has(file.path)}
                    collapsed={collapsedFiles.has(file.path)}
                    onToggleViewed={() =>
                      setViewedFiles((prev) => {
                        const next = new Set(prev);
                        if (next.has(file.path)) next.delete(file.path);
                        else next.add(file.path);
                        return next;
                      })
                    }
                    onCollapsedChange={(collapsed) =>
                      setCollapsedFiles((prev) => {
                        const next = new Set(prev);
                        if (collapsed) next.add(file.path);
                        else next.delete(file.path);
                        return next;
                      })
                    }
                    onAddAnnotation={(ann) =>
                      handleAddAnnotation({ ...ann, filePath: file.path })
                    }
                    onDeleteAnnotation={handleDeleteAnnotation}
                    onEditAnnotation={handleEditAnnotation}
                  />
                </div>
              ))}
            </div>
          )}
        </main>

        <CritiqueCommentsPane
          annotations={feedbackAnnotations}
          open={commentsPaneOpen}
          unavailableAnnotationIds={unavailableAnnotationIds}
          onOpenChange={setCommentsPaneOpen}
          onDeleteAnnotation={handleDeleteFeedbackAnnotation}
          onEditAnnotation={handleEditAnnotation}
          onJumpToAnnotation={handleJumpToAnnotation}
        />
      </div>

      <FeedbackBar
        annotationCount={feedbackAnnotations.length}
        copied={copied}
        onSubmit={handleSubmit}
        onCopyMarkdown={handleCopyMarkdown}
      />
    </div>
  );
}
