# mikrogit

Native cross-platform Git client (Linux + Windows) with a VS Code-like Source Control experience — **without Electron**. Built with [Tauri v2](https://tauri.app) (Rust backend + OS WebView) for fast startup, low RAM, and small binaries.

## Features

- Open repo / remembers last repo (or launch straight into one with `MIKROGIT_REPO=/path/to/repo`)
- **Changes**: Staged / Changes / Untracked / Merge groups — as a flat list or collapsible **folder tree** (☰/⊞ toggle); stage, unstage & discard per file **or per folder**; stage-all
- **Merge conflicts**: built-in conflict editor — side-by-side Current vs Incoming per region with *Accept Current / Incoming / Both*, inline custom edit, progress counter, **Save & stage** when fully resolved
- **Explorer**: file-manager view of the whole working tree (ignored files included, `.git` hidden) — lazy-loaded folders, file-type icons, git status badges (M/U/D), filter box, new file/folder, rename, delete; click a text file to **edit it inline** with ● dirty marker, Save (Ctrl+S), confirmation before discarding unsaved edits, read-only fallback for truncated/large and binary files
- **Diff**: inline + side-by-side, word-safe line rendering, `+`/`−` counts, untracked preview, binary detection
- **Terminal**: type raw git commands (history via ↑/↓) and watch every git operation the app runs — stage, commit, push… — with stdout/stderr and exit codes (refresh polling is filtered out)
- **Commit**: message box (Ctrl+Enter), amend
- **Sync**: fetch / pull / push / sync (pull-then-push), ahead/behind badges
- **Branches**: list local + remote, checkout, create, delete (force fallback)
- **Stash**: create (incl. untracked), apply, pop, drop
- **History**: 200-commit list, click to see files + per-file diff, load-more
- **Status bar**: branch, ahead/behind, staged/modified/untracked counts
- Auto-refresh via filesystem watcher (300 ms debounce) — explorer tree and open file list stay in sync too

## Prerequisites

- `git >= 2.40` on PATH (all git ops shell out to the CLI — hooks, LFS, credential helpers work as usual)
- Node 22 + pnpm 9+
- Rust stable
- **Ubuntu/Debian**: `sudo apt install -y libwebkit2gtk-4.1-dev build-essential libssl-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev`
- **Windows**: VS 2022 Build Tools (Desktop C++ workload), WebView2 (preinstalled on Win 10/11), Git for Windows

## Develop

```bash
pnpm install
pnpm tauri dev        # app with hot reload
./verify.sh           # full suite: frontend tests, typecheck, Rust unit +
                      # integration tests, UI E2E (auto-rebuilds binary if stale)
./verify.sh --fast    # skip E2E / rebuild
./verify.sh --watch   # re-run the fast suite on every file change
./verify.sh --build   # also rebuild the installable bundle afterwards
```

## Testing

`./verify.sh` runs everything in one command (see *Develop* above). The suite:

1. Frontend unit tests (vitest)
2. Typecheck (svelte-check)
3. Rust unit tests
4. Rust integration tests — real git repos in temp dirs: full status/diff/stage/commit flow, a real conflicting merge resolved end-to-end, explorer file ops
5. UI E2E (default; `--fast` skips) — launches the real app binary and drives the window through WebDriver: open repo → stage → commit → explorer preview → terminal

E2E prerequisites:

```bash
cargo install tauri-driver --locked
```

```bash
# Linux
sudo apt install -y webkit2gtk-driver      # provides WebKitWebDriver
```

```powershell
# Windows 11 (WebView2 → msedgedriver) — full walkthrough in docs/windows.md
```

The E2E uses the `MIKROGIT_REPO` env var to auto-open a fixture repo on launch and `data-testid` hooks in the UI for stable selectors. The Rust integration tests and the fast suite run natively on Windows too — see [docs/windows.md](docs/windows.md) for the complete VM setup.

## Build installers

```bash
pnpm tauri build -- --bundles deb,appimage   # Linux
pnpm tauri build -- --bundles nsis,msi       # Windows
```

## Project layout

```
src/                      SvelteKit frontend (SPA, adapter-static)
  lib/types.ts            shared TS types (mirror of Rust serde structs)
  lib/gitClient.ts        typed Tauri invoke() wrappers
  lib/store.ts            svelte stores + refresh orchestration
  lib/diff.ts             unified-diff parser (+ tests)
  lib/fileTree.ts         changes -> folder tree builder (+ tests)
  lib/mergeResolve.ts     conflict choice applier (+ tests)
  components/             SourceControl, FileGroup, FileTree, DiffView,
                          CommitBox, BranchPanel, HistoryPanel, StashPanel,
                          ExplorerPanel, MergeEditor, TerminalPanel, StatusBar
src-tauri/
  src/git/mod.rs          run_git() via tokio::process (GIT_PAGER=cat, no prompts)
  src/git/status.rs       porcelain=v2 parser (+ tests)
  src/git/diff.rs         diff/numstat/show parsers, untracked preview
  src/git/ops.rs          stage/commit/sync/branch/stash/log/commit-files
  src/git/files.rs        explorer: fs listing, read/save, create/rename/delete,
                          conflict-marker parser/resolver (+ tests)
  src/git/console.rs      command log (whitelisted ops -> "git-log" event, + tests)
  src/commands.rs         Tauri IPC commands (one per UI action)
  src/state.rs            AppState { open repo }
  src/watcher.rs          notify debounced watcher -> "repo-changed" event
e2e/
  run.sh                  fixture repo + tauri-driver launch + cleanup
  app.e2e.js              WebDriverIO script driving the real window
verify.sh                 one-command full test suite (see Testing)
```

## Performance notes (why not Electron)

- No bundled Chromium / Node runtime — OS WebView only (~10 MB binaries).
- Rust spawns `git` async with `GIT_OPTIONAL_LOCKS=0`; watcher debounced at 300 ms.
- Targets: cold start < 800 ms, status refresh < 200 ms on 5k-file repo, idle RAM < 120 MB.

## Out of scope for v1

Rebase workflows, blame, full commit graph, remotes/tags manager, PR hosting integration.
