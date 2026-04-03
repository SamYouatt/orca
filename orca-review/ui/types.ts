export type DiffType = "uncommitted" | "branch";

export interface DiffData {
  rawPatch: string;
  gitRef: string;
  diffType: DiffType;
  defaultBranch: string;
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
