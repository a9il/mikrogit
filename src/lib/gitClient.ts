import { invoke } from "@tauri-apps/api/core";
import type {
  Branch,
  CommitEntry,
  ConflictFile,
  FileDiff,
  RepoStatus,
  StashEntry,
  StatusBarInfo,
  TreeEntry,
} from "./types";

export const git = {
  openRepo(path: string): Promise<string> {
    return invoke("open_repo", { path });
  },
  closeRepo(): Promise<void> {
    return invoke("close_repo");
  },
  currentRepo(): Promise<string | null> {
    return invoke("current_repo");
  },
  getStatus(): Promise<RepoStatus> {
    return invoke("get_status");
  },
  getDiff(path: string, staged: boolean): Promise<FileDiff> {
    return invoke("get_diff", { path, staged });
  },
  stage(paths: string[]): Promise<void> {
    return invoke("stage", { paths });
  },
  stageAll(): Promise<void> {
    return invoke("stage_all");
  },
  unstage(paths: string[]): Promise<void> {
    return invoke("unstage", { paths });
  },
  unstageAll(): Promise<void> {
    return invoke("unstage_all");
  },
  discard(paths: string[]): Promise<void> {
    return invoke("discard", { paths });
  },
  clean(paths: string[]): Promise<void> {
    return invoke("clean", { paths });
  },
  commit(message: string, amend: boolean): Promise<string> {
    return invoke("commit", { message, amend });
  },
  fetch(remote?: string): Promise<string> {
    return invoke("fetch", { remote: remote ?? null });
  },
  pull(): Promise<string> {
    return invoke("pull");
  },
  push(setUpstream = false): Promise<string> {
    return invoke("push", { setUpstream });
  },
  sync(): Promise<string> {
    return invoke("sync");
  },
  listBranches(): Promise<Branch[]> {
    return invoke("list_branches");
  },
  checkout(name: string): Promise<void> {
    return invoke("checkout", { name });
  },
  createBranch(name: string, checkout = true): Promise<void> {
    return invoke("create_branch", { name, checkout });
  },
  deleteBranch(name: string, force = false): Promise<void> {
    return invoke("delete_branch", { name, force });
  },
  stashList(): Promise<StashEntry[]> {
    return invoke("stash_list");
  },
  stashCreate(message?: string, includeUntracked = false): Promise<void> {
    return invoke("stash_create", {
      message: message ?? null,
      includeUntracked,
    });
  },
  stashApply(index: number): Promise<void> {
    return invoke("stash_apply", { index });
  },
  stashPop(index: number): Promise<void> {
    return invoke("stash_pop", { index });
  },
  stashDrop(index: number): Promise<void> {
    return invoke("stash_drop", { index });
  },
  getLog(limit: number, skip = 0): Promise<CommitEntry[]> {
    return invoke("get_log", { limit, skip });
  },
  getCommitFiles(hash: string): Promise<string[]> {
    return invoke("get_commit_files", { hash });
  },
  getCommitDiff(hash: string, path: string): Promise<FileDiff> {
    return invoke("get_commit_diff", { hash, path });
  },
  statusBar(): Promise<StatusBarInfo> {
    return invoke("status_bar");
  },
  detectGit(): Promise<string> {
    return invoke("detect_git");
  },
  listFiles(dir?: string): Promise<TreeEntry[]> {
    return invoke("list_files", { dir: dir ?? null });
  },
  readFile(path: string): Promise<string> {
    return invoke("read_file", { path });
  },
  createFile(path: string): Promise<void> {
    return invoke("create_file", { path });
  },
  createDir(path: string): Promise<void> {
    return invoke("create_dir", { path });
  },
  renameEntry(from: string, to: string): Promise<void> {
    return invoke("rename_entry", { from, to });
  },
  deleteEntry(path: string): Promise<void> {
    return invoke("delete_entry", { path });
  },
  readConflicts(path: string): Promise<ConflictFile> {
    return invoke("read_conflicts", { path });
  },
  saveFile(path: string, content: string): Promise<void> {
    return invoke("save_file", { path, content });
  },
  saveAndStage(path: string, content: string): Promise<void> {
    return invoke("save_and_stage", { path, content });
  },
  runCustom(
    args: string[],
  ): Promise<{ stdout: string; stderr: string; code: number | null }> {
    return invoke("run_custom", { args });
  },
};
