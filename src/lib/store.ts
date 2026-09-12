import { writable, derived } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { isResolved, resolvedContent } from "./mergeResolve";
import { git } from "./gitClient";
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

export const repoPath = writable<string | null>(null);
export const currentRepo = repoPath;
export const status = writable<RepoStatus | null>(null);
export const statusBar = writable<StatusBarInfo | null>(null);
export const branches = writable<Branch[]>([]);
export const stashes = writable<StashEntry[]>([]);
export const commits = writable<CommitEntry[]>([]);
export const selectedFile = writable<{ path: string; staged: boolean } | null>(
  null,
);
export const selectedDiff = writable<FileDiff | null>(null);
export const selectedCommit = writable<string | null>(null);
export const view = writable<
  "changes" | "branches" | "history" | "stashes" | "explorer" | "terminal" | "settings"
>("changes");
export const busy = writable(false);
export const error = writable<string | null>(null);
export const diffMode = writable<"inline" | "side">("inline");
export const changesLayout = writable<"list" | "tree">("tree");
export const changesGroupsCollapsed = writable<Set<string>>(new Set());

export function toggleChangesGroup(title: string) {
  changesGroupsCollapsed.update((s) => {
    const next = new Set(s);
    if (next.has(title)) next.delete(title);
    else next.add(title);
    return next;
  });
}

export const stagedCount = derived(status, ($s) => $s?.staged.length ?? 0);
export const changeCount = derived(
  status,
  ($s) =>
    ($s?.staged.length ?? 0) +
    ($s?.unstaged.length ?? 0) +
    ($s?.untracked.length ?? 0) +
    ($s?.unmerged.length ?? 0),
);

let refreshTimer: ReturnType<typeof setTimeout> | null = null;

export function scheduleRefresh(ms = 300) {
  if (refreshTimer) clearTimeout(refreshTimer);
  refreshTimer = setTimeout(() => void refresh(), ms);
}

export async function openRepo(path: string) {
  error.set(null);
  try {
    const root = await git.openRepo(path);
    repoPath.set(root);
    selectedFile.set(null);
    selectedDiff.set(null);
    await refresh();
  } catch (e) {
    error.set(String(e));
  }
}

export async function refresh(activeView?: string) {
  let path: string | null = null;
  repoPath.subscribe((p) => (path = p))();
  if (!path) return;
  busy.set(true);
  try {
    const [s, sb] = await Promise.all([git.getStatus(), git.statusBar()]);
    status.set(s);
    statusBar.set(sb);
    error.set(null);
    if (activeView === "branches") await refreshBranches();
    else if (activeView === "stashes") await refreshStashes();
    else if (activeView === "history") await refreshLog();
    else if (activeView === "explorer") await refreshExplorer();
  } catch (e) {
    error.set(String(e));
  } finally {
    busy.set(false);
  }
}

export async function refreshBranches() {
  try {
    branches.set(await git.listBranches());
  } catch (e) {
    error.set(String(e));
  }
}

export async function refreshStashes() {
  try {
    stashes.set(await git.stashList());
  } catch (e) {
    error.set(String(e));
  }
}

export async function refreshLog() {
  try {
    commits.set(await git.getLog(200));
  } catch (e) {
    error.set(String(e));
  }
}

export const explorerEntries = writable<Record<string, TreeEntry[]>>({});
export const explorerFile = writable<string | null>(null);
export const explorerContent = writable<string | null>(null);
export const explorerDirty = writable(false);
export const conflictFile = writable<ConflictFile | null>(null);
export const conflictPath = writable<string | null>(null);
export const conflictChoices = writable<Record<number, string>>({});
export const explorerError = writable<string | null>(null);
export const explorerCollapsed = writable<Set<string>>(new Set());
export const explorerLoaded = writable<Set<string>>(new Set([""]));

function markExplorerLoaded(dir: string) {
  explorerLoaded.update((s) => new Set(s).add(dir));
}

function unmarkExplorerLoaded(dir: string) {
  explorerLoaded.update((s) => {
    const next = new Set(s);
    next.delete(dir);
    return next;
  });
}

export function explorerToggle(dir: string) {
  explorerCollapsed.update((s) => {
    const next = new Set(s);
    if (next.has(dir)) next.delete(dir);
    else next.add(dir);
    return next;
  });
}

export function explorerExpand(dir: string) {
  explorerCollapsed.update((s) => {
    if (!s.has(dir)) return s;
    const next = new Set(s);
    next.delete(dir);
    return next;
  });
}

export function explorerCollapse(dir: string) {
  explorerCollapsed.update((s) => new Set(s).add(dir));
}

export async function refreshExplorer() {
  explorerError.set(null);
  try {
    const root = await git.listFiles();
    // Preserve collapsed/loaded sets: prune entries for folders that vanished.
    const names = new Set(root.map((e) => e.path));
    explorerEntries.update((m) => {
      const next: Record<string, TreeEntry[]> = { "": root };
      for (const [k, v] of Object.entries(m)) {
        if (k && (names.has(k) || k.split("/").length > 1)) next[k] = v;
      }
      return next;
    });
    explorerCollapsed.update((s) => {
      const next = new Set<string>();
      for (const p of s) if (names.has(p)) next.add(p);
      return next;
    });
    explorerLoaded.update((s) => {
      const next = new Set<string>([""]);
      for (const p of s) if (p && names.has(p.split("/")[0])) next.add(p);
      return next;
    });
    if (root.length === 0) explorerError.set("Folder is empty or unreadable.");
  } catch (e) {
    explorerError.set(`Could not list files: ${String(e)}`);
  }
}

export async function expandExplorerDir(dir: string) {
  try {
    const entries = await git.listFiles(dir);
    explorerError.set(null);
    explorerEntries.update((m) => ({ ...m, [dir]: entries }));
    markExplorerLoaded(dir);
  } catch (e) {
    explorerError.set(`Could not open folder ${dir}: ${String(e)}`);
    throw e;
  }
}

export async function reloadExplorerDir(dir: string) {
  try {
    const entries = await git.listFiles(dir === "" ? undefined : dir);
    explorerEntries.update((m) => ({ ...m, [dir]: entries }));
    markExplorerLoaded(dir);
  } catch (e) {
    error.set(String(e));
  }
}

export async function explorerCreate(kind: "file" | "dir", dir: string, name: string) {
  const clean = name.trim().replace(/\\/g, "/");
  if (!clean) return;
  const target = dir ? `${dir}/${clean}` : clean;
  try {
    if (kind === "file") await git.createFile(target);
    else await git.createDir(target);
    await reloadExplorerDir(dir);
    if (kind === "file") await openExplorerFile(target);
  } catch (e) {
    error.set(String(e));
  }
}

export async function explorerRename(from: string, to: string, parentDir: string) {
  try {
    await git.renameEntry(from, to);
    explorerEntries.set({});
    explorerLoaded.set(new Set([""]));
    explorerCollapsed.set(new Set());
    await reloadExplorerDir("");
    if (parentDir) await reloadExplorerDir(parentDir);
  } catch (e) {
    error.set(String(e));
  }
}

export async function explorerDelete(path: string, parentDir: string) {
  try {
    await git.deleteEntry(path);
    if (parentDir) await reloadExplorerDir(parentDir);
    else await reloadExplorerDir("");
    let cur: string | null = null;
    explorerFile.subscribe((p) => (cur = p))();
    if (cur === path) {
      explorerFile.set(null);
      explorerContent.set(null);
      explorerDirty.set(false);
    }
  } catch (e) {
    error.set(String(e));
  }
}

export async function loadConflicts(path: string) {
  conflictPath.set(path);
  conflictFile.set(null);
  conflictChoices.set({});
  try {
    const cf = await git.readConflicts(path);
    conflictFile.set(cf);
  } catch (e) {
    error.set(String(e));
  }
}

export function setConflictChoice(regionIdx: number, choice: string) {
  conflictChoices.update((c) => ({ ...c, [regionIdx]: choice }));
}

export function clearConflictChoice(regionIdx: number) {
  conflictChoices.update((c) => {
    const next = { ...c };
    delete next[regionIdx];
    return next;
  });
}

function readConflictState(): [ConflictFile | null, Record<number, string>, string | null] {
  let cf: ConflictFile | null = null;
  let choices: Record<number, string> = {};
  let path: string | null = null;
  conflictFile.subscribe((v) => { cf = v; })();
  conflictChoices.subscribe((v) => { choices = v; })();
  conflictPath.subscribe((v) => { path = v; })();
  return [cf, choices, path];
}

export async function saveResolvedConflicts() {
  const [cf, choices, path] = readConflictState();
  if (!cf || !path) return;
  let allResolved = true;
  const args: string[] = [];
  for (let idx = 0; idx < cf.regions.length; idx++) {
    const choice = choices[idx] ?? "";
    if (!isResolved(choice)) {
      allResolved = false;
    }
    args.push(choice);
  }
  if (!allResolved) {
    error.set("Resolve all conflicts before saving.");
    return;
  }
  try {
    await git.saveFile(path, resolvedContent(cf, args));
    await git.stage([path]);
    conflictFile.set(null);
    conflictPath.set(null);
    conflictChoices.set({});
    await refresh();
  } catch (e) {
    error.set(String(e));
  }
}

export async function openExplorerFile(path: string) {
  let dirty = false;
  explorerDirty.subscribe((d) => (dirty = d))();
  if (dirty && !confirm("Discard unsaved changes?")) return;
  explorerFile.set(path);
  explorerContent.set(null);
  explorerDirty.set(false);
  try {
    explorerContent.set(await git.readFile(path));
  } catch (e) {
    error.set(String(e));
  }
}

export async function saveExplorerFile() {
  let path: string | null = null;
  let content: string | null = null;
  explorerFile.subscribe((p) => (path = p))();
  explorerContent.subscribe((c) => (content = c))();
  if (!path || content === null) return;
  const p = path;
  const c = content;
  try {
    await git.saveFile(p, c);
    explorerDirty.set(false);
    await refresh();
  } catch (e) {
    error.set(String(e));
  }
}

export async function loadDiff(path: string, staged: boolean) {
  selectedFile.set({ path, staged });
  selectedDiff.set(null);
  try {
    selectedDiff.set(await git.getDiff(path, staged));
  } catch (e) {
    error.set(String(e));
  }
}

export async function runOp<T>(fn: () => Promise<T>): Promise<T | null> {
  busy.set(true);
  error.set(null);
  try {
    const r = await fn();
    await refresh();
    return r;
  } catch (e) {
    error.set(String(e));
    return null;
  } finally {
    busy.set(false);
  }
}

export function initWatcher() {
  void listen<string>("repo-changed", () => {
    scheduleRefresh();
    void refreshExplorerQuiet();
  });
  void listen<string>("repo-opened", (e) => {
    repoPath.set(e.payload);
    void refresh();
    void refreshExplorer();
  });
}

export interface ConsoleEntry {
  cmd: string;
  stdout: string;
  stderr: string;
  code: number | null;
  ts: number;
}

export const consoleEntries = writable<ConsoleEntry[]>([]);

export function initConsole() {
  void listen<ConsoleEntry>("git-log", (e) => {
    consoleEntries.update((l) => {
      const next = [...l, e.payload];
      return next.length > 500 ? next.slice(next.length - 500) : next;
    });
  });
}

export function splitArgs(input: string): string[] {
  const out: string[] = [];
  const re = /"([^"]*)"|'([^']*)'|(\S+)/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(input))) out.push(m[1] ?? m[2] ?? m[3]);
  return out;
}

export async function runConsoleCommand(cmdline: string) {
  let args = splitArgs(cmdline);
  if (args[0]?.toLowerCase() === "git") args = args.slice(1);
  if (args.length === 0) return;
  try {
    await git.runCustom(args);
    await refresh();
  } catch (e) {
    consoleEntries.update((l) => [
      ...l,
      {
        cmd: `git ${args.join(" ")}`,
        stdout: "",
        stderr: String(e),
        code: null,
        ts: Date.now(),
      },
    ]);
  }
}

async function refreshExplorerQuiet() {
  try {
    let entries: Record<string, TreeEntry[]> = {};
    explorerEntries.subscribe((m) => (entries = m))();
    const dirs = Object.keys(entries);
    if (dirs.length === 0) return;
    const root = await git.listFiles();
    explorerEntries.update((m) => ({ ...m, "": root }));
    for (const d of dirs) {
      if (!d) continue;
      try {
        const sub = await git.listFiles(d);
        explorerEntries.update((m) => ({ ...m, [d]: sub }));
      } catch {
        // Folder may have been deleted; drop its stale cache.
        explorerEntries.update((m) => {
          const next = { ...m };
          delete next[d];
          return next;
        });
        unmarkExplorerLoaded(d);
      }
    }
  } catch {
    // Leave existing entries in place on transient failure.
  }
}
