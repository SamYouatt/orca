import React, { useState, useEffect, useCallback, useRef, useMemo } from "react";
import "./app.css";
import type { Annotation } from "./types";
import { DiffToggle } from "./components/DiffToggle";
import { FileList } from "./components/FileList";
import { DiffViewer } from "./components/DiffViewer";
import { FeedbackBar } from "./components/FeedbackBar";
import { useTheme } from "./hooks/useTheme";

interface ServerFileContents {
  path: string;
  oldContent: string | null;
  newContent: string | null;
}

interface DiffFile {
  path: string;
  oldPath?: string;
  patch: string;
  additions: number;
  deletions: number;
  oldContent?: string | null;
  newContent?: string | null;
}

interface DiffState {
  rawPatch: string;
  gitRef: string;
  diffType: "uncommitted" | "branch";
  defaultBranch: string;
  files: ServerFileContents[];
  error?: string;
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

export default function App() {
  const [diff, setDiff] = useState<DiffState | null>(null);
  const [files, setFiles] = useState<DiffFile[]>([]);
  const [activeFile, setActiveFile] = useState<string | null>(null);
  const [annotations, setAnnotations] = useState<Annotation[]>([]);
  const [switching, setSwitching] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [viewedFiles, setViewedFiles] = useState<Set<string>>(new Set());
  const fileRefs = useRef<Map<string, HTMLDivElement>>(new Map());
  const theme = useTheme();

  useEffect(() => {
    fetch("/api/diff")
      .then((res) => res.json())
      .then((data: DiffState) => {
        setDiff(data);
        const parsed = parseDiffToFiles(data.rawPatch, data.files || []);
        setFiles(parsed);
        if (parsed.length > 0) setActiveFile(parsed[0].path);
      });
  }, []);

  const handleSwitch = useCallback(
    async (diffType: "uncommitted" | "branch") => {
      setSwitching(true);
      try {
        const res = await fetch("/api/diff/switch", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ diffType }),
        });
        const data: DiffState = await res.json();
        setDiff(data);
        const parsed = parseDiffToFiles(data.rawPatch, data.files || []);
        setFiles(parsed);
        setAnnotations([]);
        setViewedFiles(new Set());
        setActiveFile(parsed.length > 0 ? parsed[0].path : null);
      } finally {
        setSwitching(false);
      }
    },
    []
  );

  const handleAddAnnotation = useCallback(
    (ann: Omit<Annotation, "id">) => {
      setAnnotations((prev) => [
        ...prev,
        { ...ann, id: crypto.randomUUID() },
      ]);
    },
    []
  );

  const handleDeleteAnnotation = useCallback((id: string) => {
    setAnnotations((prev) => prev.filter((a) => a.id !== id));
  }, []);

  const handleEditAnnotation = useCallback((id: string, text: string) => {
    setAnnotations((prev) =>
      prev.map((a) => (a.id === id ? { ...a, text } : a))
    );
  }, []);

  const buildMarkdown = useCallback(() => {
    if (annotations.length === 0) return "Code review completed — no changes requested.";

    const parts: string[] = ["# Code Review Feedback"];
    const grouped = new Map<string, Annotation[]>();
    for (const ann of annotations) {
      const existing = grouped.get(ann.filePath) || [];
      existing.push(ann);
      grouped.set(ann.filePath, existing);
    }

    for (const [filePath, fileAnns] of grouped) {
      parts.push(`## ${filePath}`);
      const sorted = [...fileAnns].sort((a, b) => a.lineStart - b.lineStart);
      for (const ann of sorted) {
        const range =
          ann.lineStart === ann.lineEnd
            ? `Line ${ann.lineStart}`
            : `Lines ${ann.lineStart}-${ann.lineEnd}`;
        parts.push(`### ${range} (${ann.side})\n${ann.text}`);
      }
    }

    parts.push("Address all feedback above.");
    return parts.join("\n\n");
  }, [annotations]);

  const handleSubmit = useCallback(async () => {
    await fetch("/api/feedback", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ overallComment: "", annotations }),
    });
    setSubmitted(true);
  }, [annotations]);

  const [copied, setCopied] = useState(false);
  const handleCopyMarkdown = useCallback(async () => {
    navigator.clipboard.writeText(buildMarkdown());
    setCopied(true);
    await fetch("/api/feedback", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ overallComment: "", annotations }),
    });
    setSubmitted(true);
  }, [buildMarkdown, annotations]);

  const scrollToFile = useCallback((filePath: string) => {
    setActiveFile(filePath);
    const el = fileRefs.current.get(filePath);
    if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
  }, []);

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
          switching={switching}
          onSwitch={handleSwitch}
        />
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col">
      <header className="flex items-center justify-between px-4 py-2 border-b bg-card">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-sm">Orca critique</span>
        </div>
        <DiffToggle
          current={diff.diffType}
          defaultBranch={diff.defaultBranch}
          switching={switching}
          onSwitch={handleSwitch}
        />
      </header>

      <div className="flex flex-1 overflow-hidden">
        <aside className="w-64 border-r bg-card overflow-y-auto shrink-0">
          <FileList
            files={files}
            activeFile={activeFile}
            annotations={annotations}
            onSelect={scrollToFile}
          />
        </aside>

        <main className="flex-1 overflow-y-auto p-4 bg-muted">
          {files.length === 0 ? (
            <div className="text-muted-foreground text-center mt-20">
              No changes to review.
            </div>
          ) : (
            <div className="flex flex-col gap-4">
              {files.map((file) => (
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
                    themeType={theme}
                    viewed={viewedFiles.has(file.path)}
                    onToggleViewed={() =>
                      setViewedFiles((prev) => {
                        const next = new Set(prev);
                        if (next.has(file.path)) next.delete(file.path);
                        else next.add(file.path);
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
      </div>

      <FeedbackBar
        annotationCount={annotations.length}
        copied={copied}
        onSubmit={handleSubmit}
        onCopyMarkdown={handleCopyMarkdown}
      />
    </div>
  );
}
