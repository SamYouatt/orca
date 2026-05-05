import React from "react";
import { IconChevronSm } from "@pierre/icons";
import type { Annotation } from "../types";
import type { TreeFile, TreeNode } from "../lib/fileTree";
import { buildFileTree } from "../lib/fileTree";
import { Badge } from "@/components/ui/badge";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";

const INDENT_PX = 16;
const ROW_BASE_PX = 8;

interface FileTreeProps {
  files: TreeFile[];
  activeFile: string | null;
  annotations: Annotation[];
  collapsed: Set<string>;
  onToggleDir: (path: string) => void;
  onSelectFile: (path: string) => void;
}

export function FileTree({
  files,
  activeFile,
  annotations,
  collapsed,
  onToggleDir,
  onSelectFile,
}: FileTreeProps) {
  const tree = React.useMemo(() => buildFileTree(files), [files]);
  return (
    <div className="py-2">
      {tree.map((node) => (
        <TreeRow
          key={nodeKey(node)}
          node={node}
          depth={0}
          activeFile={activeFile}
          annotations={annotations}
          collapsed={collapsed}
          onToggleDir={onToggleDir}
          onSelectFile={onSelectFile}
        />
      ))}
    </div>
  );
}

interface TreeRowProps {
  node: TreeNode;
  depth: number;
  activeFile: string | null;
  annotations: Annotation[];
  collapsed: Set<string>;
  onToggleDir: (path: string) => void;
  onSelectFile: (path: string) => void;
}

function TreeRow({
  node,
  depth,
  activeFile,
  annotations,
  collapsed,
  onToggleDir,
  onSelectFile,
}: TreeRowProps) {
  if (node.kind === "file") {
    return (
      <FileRow
        file={node.file}
        depth={depth}
        active={activeFile === node.file.path}
        annotationCount={
          annotations.filter((a) => a.filePath === node.file.path).length
        }
        onSelect={onSelectFile}
      />
    );
  }
  const isOpen = !collapsed.has(node.path);
  return (
    <Collapsible
      open={isOpen}
      onOpenChange={() => onToggleDir(node.path)}
    >
      <CollapsibleTrigger
        className="w-full flex items-center text-left text-sm font-normal hover:bg-muted py-1.5 pr-3 cursor-pointer"
        style={{ paddingLeft: depth * INDENT_PX + ROW_BASE_PX }}
      >
        <IconChevronSm
          className={`shrink-0 size-4 transition-transform ${
            isOpen ? "" : "-rotate-90"
          }`}
        />
        <span className="truncate flex-1 ml-0.5">
          {node.segments.join("/")}
        </span>
      </CollapsibleTrigger>
      <CollapsibleContent>
        {node.children.map((child) => (
          <TreeRow
            key={nodeKey(child)}
            node={child}
            depth={depth + 1}
            activeFile={activeFile}
            annotations={annotations}
            collapsed={collapsed}
            onToggleDir={onToggleDir}
            onSelectFile={onSelectFile}
          />
        ))}
      </CollapsibleContent>
    </Collapsible>
  );
}

interface FileRowProps {
  file: TreeFile;
  depth: number;
  active: boolean;
  annotationCount: number;
  onSelect: (path: string) => void;
}

function FileRow({ file, depth, active, annotationCount, onSelect }: FileRowProps) {
  const name = file.path.split("/").pop() ?? file.path;
  return (
    <button
      type="button"
      onClick={() => onSelect(file.path)}
      className={`w-full flex items-center text-left text-sm font-normal hover:bg-muted py-1.5 pr-3 cursor-pointer ${
        active ? "bg-muted" : ""
      }`}
      style={{ paddingLeft: depth * INDENT_PX + ROW_BASE_PX }}
    >
      <span className="shrink-0 size-4" aria-hidden="true" />
      <span className="truncate flex-1 ml-0.5">{name}</span>
      <span className="flex items-center gap-1 text-xs shrink-0">
        {file.additions > 0 && (
          <span className="text-green-600 dark:text-green-400">+{file.additions}</span>
        )}
        {file.deletions > 0 && (
          <span className="text-destructive">-{file.deletions}</span>
        )}
        {annotationCount > 0 && (
          <Badge className="rounded-full w-4 h-4 p-0 text-[10px] font-bold ml-1 bg-blue-500 text-white">
            {annotationCount}
          </Badge>
        )}
      </span>
    </button>
  );
}

function nodeKey(node: TreeNode): string {
  return node.kind === "dir" ? `d:${node.path}` : `f:${node.file.path}`;
}
