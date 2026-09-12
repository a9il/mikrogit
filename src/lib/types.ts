export type FileStatus =
  | "modified"
  | "added"
  | "deleted"
  | "renamed"
  | "copied"
  | "untracked"
  | "ignored"
  | "unmerged";

export interface FileEntry {
  path: string;
  original_path?: string;
  index_status: FileStatus | null;
  worktree_status: FileStatus | null;
}

export interface BranchInfo {
  name: string | null;
  upstream: string | null;
  ahead: number;
  behind: number;
  detached: boolean;
}

export interface RepoStatus {
  branch: BranchInfo;
  staged: FileEntry[];
  unstaged: FileEntry[];
  untracked: FileEntry[];
  unmerged: FileEntry[];
}

export interface DiffHunk {
  header: string;
  old_start: number;
  old_lines: number;
  new_start: number;
  new_lines: number;
  lines: DiffLine[];
}

export interface DiffLine {
  kind: "context" | "add" | "del" | "hunk" | "file";
  content: string;
  old_no?: number;
  new_no?: number;
}

export interface FileDiff {
  path: string;
  old_path?: string;
  hunks: DiffHunk[];
  binary: boolean;
  added: number;
  removed: number;
}

export interface Branch {
  name: string;
  current: boolean;
  remote: boolean;
  upstream?: string;
  ahead: number;
  behind: number;
  last_commit: string;
  last_commit_subject: string;
}

export interface StashEntry {
  index: number;
  message: string;
  branch: string;
}

export interface CommitEntry {
  hash: string;
  short_hash: string;
  author: string;
  email: string;
  date: string;
  subject: string;
  body: string;
  parents: string[];
  refs: string[];
}

export interface StatusBarInfo {
  branch: string | null;
  ahead: number;
  behind: number;
  staged_count: number;
  unstaged_count: number;
  untracked_count: number;
  sync_state: "idle" | "in-progress";
}

export interface TreeEntry {
  path: string;
  kind: string;
  size: number;
}

export interface ConflictRegion {
  current_header: string;
  current_lines: string[];
  incoming_header: string;
  incoming_lines: string[];
}

export interface ConflictFile {
  path: string;
  regions: ConflictRegion[];
  has_conflicts: boolean;
  original_content: string;
}

