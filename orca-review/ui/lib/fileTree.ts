export interface TreeFile {
  path: string;
  additions: number;
  deletions: number;
}

export type TreeNode =
  | { kind: "dir"; path: string; segments: string[]; children: TreeNode[] }
  | { kind: "file"; file: TreeFile };

interface DirBuilder {
  path: string;
  segments: string[];
  childDirs: Map<string, DirBuilder>;
  childFiles: TreeFile[];
}

function makeDir(path: string, segments: string[]): DirBuilder {
  return { path, segments, childDirs: new Map(), childFiles: [] };
}

function buildRaw(files: TreeFile[]): DirBuilder {
  const root = makeDir("", []);
  for (const file of files) {
    const parts = file.path.split("/");
    let cursor = root;
    for (let i = 0; i < parts.length - 1; i++) {
      const segment = parts[i];
      let child = cursor.childDirs.get(segment);
      if (!child) {
        const childPath = cursor.path === "" ? segment : `${cursor.path}/${segment}`;
        child = makeDir(childPath, [segment]);
        cursor.childDirs.set(segment, child);
      }
      cursor = child;
    }
    cursor.childFiles.push(file);
  }
  return root;
}

function sortChildren(nodes: TreeNode[]): TreeNode[] {
  const dirs: TreeNode[] = [];
  const files: TreeNode[] = [];
  for (const node of nodes) {
    if (node.kind === "dir") dirs.push(node);
    else files.push(node);
  }
  dirs.sort((a, b) => {
    if (a.kind !== "dir" || b.kind !== "dir") return 0;
    return a.segments.join("/").toLowerCase().localeCompare(b.segments.join("/").toLowerCase());
  });
  files.sort((a, b) => {
    if (a.kind !== "file" || b.kind !== "file") return 0;
    const aName = a.file.path.split("/").pop()!.toLowerCase();
    const bName = b.file.path.split("/").pop()!.toLowerCase();
    return aName.localeCompare(bName);
  });
  return [...dirs, ...files];
}

function compact(dir: DirBuilder): DirBuilder {
  for (const [key, child] of dir.childDirs) {
    const compactedChild = compact(child);
    dir.childDirs.set(key, compactedChild);
  }
  if (dir.childFiles.length === 0 && dir.childDirs.size === 1) {
    const [onlyChild] = dir.childDirs.values();
    return {
      path: onlyChild.path,
      segments: [...dir.segments, ...onlyChild.segments],
      childDirs: onlyChild.childDirs,
      childFiles: onlyChild.childFiles,
    };
  }
  return dir;
}

function toTreeNodes(dir: DirBuilder): TreeNode[] {
  const children: TreeNode[] = [];
  for (const child of dir.childDirs.values()) {
    children.push({
      kind: "dir",
      path: child.path,
      segments: child.segments,
      children: toTreeNodes(child),
    });
  }
  for (const file of dir.childFiles) {
    children.push({ kind: "file", file });
  }
  return sortChildren(children);
}

export function buildFileTree(files: TreeFile[]): TreeNode[] {
  const raw = buildRaw(files);
  for (const [key, child] of raw.childDirs) {
    raw.childDirs.set(key, compact(child));
  }
  return toTreeNodes(raw);
}

export function countFileDescendants(node: TreeNode): number {
  if (node.kind === "file") return 1;
  let total = 0;
  for (const child of node.children) total += countFileDescendants(child);
  return total;
}

export function flattenTreeFiles(nodes: TreeNode[]): TreeFile[] {
  const out: TreeFile[] = [];
  for (const node of nodes) {
    if (node.kind === "file") out.push(node.file);
    else out.push(...flattenTreeFiles(node.children));
  }
  return out;
}
