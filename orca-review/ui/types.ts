export type DiffType = "uncommitted" | "branch" | "commit";

export interface CommitOption {
  sha: string;
  shortSha: string;
  subject: string;
}

export interface ServerFileContents {
  path: string;
  oldContent: string | null;
  newContent: string | null;
}

export interface DiffData {
  rawPatch: string;
  gitRef: string;
  diffType: DiffType;
  defaultBranch: string;
  commitOptions: CommitOption[];
  selectedCommit?: CommitOption;
  files: ServerFileContents[];
  error?: string;
}

export interface Annotation {
  id: string;
  filePath: string;
  side: "additions" | "deletions";
  lineStart: number;
  lineEnd: number;
  text: string;
}

export interface FeedbackPayload {
  overallComment: string;
  annotations: Annotation[];
}
