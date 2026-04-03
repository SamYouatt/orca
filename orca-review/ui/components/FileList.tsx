import React from "react";
import type { Annotation } from "../types";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

interface FileListProps {
  files: { path: string; additions: number; deletions: number }[];
  activeFile: string | null;
  annotations: Annotation[];
  onSelect: (path: string) => void;
}

export function FileList({ files, activeFile, annotations, onSelect }: FileListProps) {
  return (
    <div className="py-2">
      <div className="px-3 py-1 text-xs font-semibold text-muted-foreground tracking-wider">
        Files ({files.length})
      </div>
      {files.map((file) => {
        const count = annotations.filter((a) => a.filePath === file.path).length;
        return (
          <Button
            key={file.path}
            variant="ghost"
            className={`w-full justify-start rounded-none px-3 py-1.5 text-sm font-normal h-auto ${
              activeFile === file.path ? "bg-muted" : ""
            }`}
            onClick={() => onSelect(file.path)}
          >
            <span className="truncate flex-1 text-left">{file.path}</span>
            <span className="flex items-center gap-1 text-xs shrink-0">
              {file.additions > 0 && (
                <span className="text-green-600 dark:text-green-400">+{file.additions}</span>
              )}
              {file.deletions > 0 && (
                <span className="text-destructive">-{file.deletions}</span>
              )}
              {count > 0 && (
                <Badge className="rounded-full w-4 h-4 p-0 text-[10px] font-bold ml-1">
                  {count}
                </Badge>
              )}
            </span>
          </Button>
        );
      })}
    </div>
  );
}
