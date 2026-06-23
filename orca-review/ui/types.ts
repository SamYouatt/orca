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
  currentBranch: string;
  defaultBranch: string;
  commitOptions: CommitOption[];
  selectedCommit?: CommitOption;
  files: ServerFileContents[];
  error?: string;
}

export type AnnotationOrigin =
  | {
      type: "uncommitted";
      currentBranch: string;
    }
  | {
      type: "branch";
      currentBranch: string;
      defaultBranch: string;
    }
  | {
      type: "commit";
      currentBranch: string;
      commit: CommitOption;
    };

export interface Annotation {
  id: string;
  filePath: string;
  side: "additions" | "deletions";
  lineStart: number;
  lineEnd: number;
  text: string;
  reviewScope?: string;
  createdAt?: string;
  origin?: AnnotationOrigin;
}

export type AnnotationDraft = Omit<
  Annotation,
  "id" | "createdAt" | "origin" | "reviewScope"
> &
  Pick<Annotation, "reviewScope">;
export type FeedbackAnnotation = Omit<Annotation, "id" | "createdAt" | "origin" | "reviewScope">;

export interface FeedbackPayload {
  overallComment: string;
  annotations: FeedbackAnnotation[];
}
