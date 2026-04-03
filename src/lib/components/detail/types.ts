export type FileStatus = 'A' | 'M' | 'D' | 'R' | 'C';

export interface ChangedFile {
  status: FileStatus;
  path: string;
  oldPath: string | null;
  insertions: number;
  deletions: number;
}

export interface FileDiff {
  original: string;
  modified: string;
}

export interface WorkingDirectoryStatus {
  staged: ChangedFile[];
  unstaged: ChangedFile[];
}
