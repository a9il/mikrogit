<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { git } from "$lib/gitClient";
  import {
    busy,
    changesLayout,
    conflictPath,
    error,
    explorerContent,
    explorerDirty,
    explorerFile,
    loadConflicts,
    loadDiff,
    openRepo,
    refresh,
    repoPath,
    runOp,
    saveExplorerFile,
    selectedDiff,
    selectedFile,
    status,
    statusBar,
    view,
  } from "$lib/store";
  import FileGroup from "./FileGroup.svelte";
  import FileTree from "./FileTree.svelte";
  import CommitBox from "./CommitBox.svelte";
  import DiffView from "./DiffView.svelte";
  import BranchPanel from "./BranchPanel.svelte";
  import ExplorerPanel from "./ExplorerPanel.svelte";
  import HistoryPanel from "./HistoryPanel.svelte";
  import MergeEditor from "./MergeEditor.svelte";
  import StashPanel from "./StashPanel.svelte";
  import StatusBar from "./StatusBar.svelte";
  import TerminalPanel from "./TerminalPanel.svelte";
  import type { FileEntry } from "$lib/types";

  let output = $state("");

  async function pickFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === "string") await openRepo(dir);
  }

  function selectFile(f: FileEntry, staged: boolean) {
    if (f.index_status === "unmerged" || f.worktree_status === "unmerged") {
      loadConflicts(f.path);
      selectedFile.set(null);
      selectedDiff.set(null);
      return;
    }
    loadDiff(f.path, staged);
  }

  async function syncOp(kind: "fetch" | "pull" | "push" | "sync") {
    output = "";
    const r = await runOp(() => {
      switch (kind) {
        case "fetch":
          return git.fetch();
        case "pull":
          return git.pull();
        case "push":
          return git.push();
        case "sync":
          return git.sync();
      }
    });
    if (typeof r === "string") output = r.split("\n").slice(-3).join("\n");
  }

  const selPath = $derived($selectedFile?.path ?? null);
  const staged = $derived($status?.staged ?? []);
  const unstaged = $derived($status?.unstaged ?? []);
  const untracked = $derived($status?.untracked ?? []);
  const unmerged = $derived($status?.unmerged ?? []);
</script>

<div class="h-screen flex flex-col">
  <div class="flex items-center gap-1 px-2 py-1 bg-[#2d2d2d] border-b border-black">
    <span class="font-bold text-sm text-white px-2">mikrogit</span>
    {#if $repoPath}
      <span class="text-xs text-[#999] truncate font-mono" data-testid="repo-path" title={$repoPath ?? ""}>{$repoPath}</span>
    {/if}
    <span class="ml-auto flex gap-1 text-xs">
      {#if $repoPath}
        <button class="tb-btn" title="Fetch" onclick={() => syncOp("fetch")}>⇩ fetch</button>
        <button class="tb-btn" title="Pull" onclick={() => syncOp("pull")}>↓ pull</button>
        <button class="tb-btn" title="Push" onclick={() => syncOp("push")}>↑ push</button>
        <button class="tb-btn primary" title="Sync (pull then push)" onclick={() => syncOp("sync")}>⇅ sync</button>
      {/if}
      <button class="tb-btn" onclick={pickFolder}>Open repo</button>
    </span>
  </div>

  {#if $error}
    <div class="bg-[#5a1d1d] text-[#f48771] text-xs px-3 py-1.5 flex">
      <span class="flex-1 truncate">{$error}</span>
      <button class="ml-2 underline" onclick={() => error.set(null)}>dismiss</button>
    </div>
  {/if}
  {#if output}
    <div class="bg-[#1a3319] text-[#b5cea8] text-xs px-3 py-1 font-mono whitespace-pre-wrap">{output}</div>
  {/if}
  {#if $busy}
    <div class="bg-[#094771] text-white text-xs px-3 py-0.5">Working…</div>
  {/if}

  {#if !$repoPath}
    <div class="flex-1 flex flex-col items-center justify-center gap-3">
      <div class="text-lg text-[#999]">No repository open</div>
      <button class="bg-[#0e639c] hover:bg-[#1177bb] text-white px-6 py-2 rounded" onclick={pickFolder}>
        Open a Git repository
      </button>
    </div>
  {:else}
    <div class="flex-1 flex min-h-0">
      <nav class="w-12 bg-[#333333] flex flex-col items-center py-2 gap-1 shrink-0">
        {#each [["explorer", "🗀", "Explorer"], ["changes", "⎇", "Source Control"], ["branches", "⑂", "Branches"], ["history", "◷", "History"], ["stashes", "▤", "Stash"], ["terminal", ">_", "Terminal"]] as [id, icon, tip]}
          <button
            class="nav-btn"
            class:active={$view === id}
            title={tip}
            onclick={() => view.set(id as typeof $view)}>{icon}</button
          >
        {/each}
        <button class="nav-btn mt-auto" title="Refresh" onclick={() => refresh($view)} disabled={$busy}>⟳</button>
      </nav>

      {#if $view === "terminal"}
        <div class="flex-1 min-w-0 flex">
          <TerminalPanel />
        </div>
      {:else}
      <aside class="w-[320px] shrink-0 bg-[#252526] border-r border-black overflow-auto flex flex-col">
        {#if $view === "changes"}
          <CommitBox />
          <div class="flex items-center gap-1 px-3 py-1 text-xs text-[#858585]">
            <span class="flex-1"></span>
            <button
              class="layout-btn"
              class:active={$changesLayout === "list"}
              title="List view"
              onclick={() => changesLayout.set("list")}>☰</button
            >
            <button
              class="layout-btn"
              class:active={$changesLayout === "tree"}
              title="Tree view"
              onclick={() => changesLayout.set("tree")}>⊞</button
            >
          </div>
          {#if $changesLayout === "tree"}
            {#if unmerged.length > 0}
              <FileTree
                title="Merge changes"
                files={unmerged}
                emptyHint=""
                icons={{ stage: "Stage", discard: "Discard" }}
                selectedPath={selPath}
                onSelect={(f) => selectFile(f, false)}
                onDiscard={(f) => runOp(() => git.discard([f.path]))}
              />
            {/if}
            <FileTree
              title="Staged changes"
              files={staged}
              emptyHint="No staged changes"
              icons={{ stage: "Unstage", discard: "Unstage" }}
              selectedPath={selPath}
              onSelect={(f) => selectFile(f, true)}
              onUnstage={(f) => runOp(() => git.unstage([f.path]))}
              onUnstageAll={() => runOp(() => git.unstageAll())}
              onDiscard={(f) => runOp(() => git.unstage([f.path]))}
              onUnstageDir={(_dir, fs) => runOp(() => git.unstage(fs.map((f) => f.path)))}
              onDiscardDir={(_dir, fs) => runOp(() => git.unstage(fs.map((f) => f.path)))}
            />
            <FileTree
              title="Changes"
              files={unstaged}
              emptyHint="No unstaged changes"
              icons={{ stage: "Stage", discard: "Discard changes" }}
              selectedPath={selPath}
              onSelect={(f) => selectFile(f, false)}
              onStage={(f) => runOp(() => git.stage([f.path]))}
              onStageAll={() => runOp(() => git.stageAll())}
              onDiscard={(f) => {
                if (confirm(`Discard changes to ${f.path}?`)) runOp(() => git.discard([f.path]));
              }}
              onStageDir={(_dir, fs) => runOp(() => git.stage(fs.map((f) => f.path)))}
              onDiscardDir={(dir, fs) => {
                if (confirm(`Discard changes to ${fs.length} files under ${dir}?`))
                  runOp(() => git.discard(fs.map((f) => f.path)));
              }}
            />
            <FileTree
              title="Untracked"
              files={untracked}
              emptyHint="No untracked files"
              icons={{ stage: "Stage", discard: "Delete file" }}
              selectedPath={selPath}
              onSelect={(f) => selectFile(f, false)}
              onStage={(f) => runOp(() => git.stage([f.path]))}
              onStageAll={() => runOp(() => git.stageAll())}
              onDiscard={(f) => {
                if (confirm(`Permanently delete ${f.path}?`)) runOp(() => git.clean([f.path]));
              }}
              onStageDir={(_dir, fs) => runOp(() => git.stage(fs.map((f) => f.path)))}
              onDiscardDir={(dir, fs) => {
                if (confirm(`Permanently delete ${fs.length} files under ${dir}?`))
                  runOp(() => git.clean(fs.map((f) => f.path)));
              }}
            />
          {:else}
          {#if unmerged.length > 0}
            <FileGroup
              title="Merge changes"
              files={unmerged}
              emptyHint=""
              icons={{ stage: "Stage", discard: "Discard" }}
              selectedPath={selPath}
              onSelect={(f) => selectFile(f, false)}
              onDiscard={(f) => runOp(() => git.discard([f.path]))}
            />
          {/if}
          <FileGroup
            title="Staged changes"
            files={staged}
            emptyHint="No staged changes"
            icons={{ stage: "Unstage", discard: "Unstage" }}
            selectedPath={selPath}
            onSelect={(f) => selectFile(f, true)}
            onUnstage={(f) => runOp(() => git.unstage([f.path]))}
            onUnstageAll={() => runOp(() => git.unstageAll())}
            onDiscard={(f) => runOp(() => git.unstage([f.path]))}
          />
          <FileGroup
            title="Changes"
            files={unstaged}
            emptyHint="No unstaged changes"
            icons={{ stage: "Stage", discard: "Discard changes" }}
            selectedPath={selPath}
            onSelect={(f) => selectFile(f, false)}
            onStage={(f) => runOp(() => git.stage([f.path]))}
            onStageAll={() => runOp(() => git.stageAll())}
            onDiscard={(f) => {
              if (confirm(`Discard changes to ${f.path}?`)) runOp(() => git.discard([f.path]));
            }}
          />
          <FileGroup
            title="Untracked"
            files={untracked}
            emptyHint="No untracked files"
            icons={{ stage: "Stage", discard: "Delete file" }}
            selectedPath={selPath}
            onSelect={(f) => selectFile(f, false)}
            onStage={(f) => runOp(() => git.stage([f.path]))}
            onStageAll={() => runOp(() => git.stageAll())}
            onDiscard={(f) => {
              if (confirm(`Permanently delete ${f.path}?`)) runOp(() => git.clean([f.path]));
            }}
          />
          {/if}
        {:else if $view === "branches"}
          <BranchPanel />
        {:else if $view === "history"}
          <HistoryPanel />
        {:else if $view === "stashes"}
          <StashPanel />
        {:else if $view === "explorer"}
          <ExplorerPanel />
        {/if}
      </aside>

      <main class="flex-1 flex flex-col min-w-0">
        {#if $conflictPath}
          <MergeEditor />
        {:else if $view === "explorer"}
          {#if $explorerFile}
            <div class="px-3 py-2 text-xs text-[#6e6e6e] border-b border-[#2d2d2d] flex items-center gap-2">
              <span class="font-mono flex-1 truncate">{$explorerFile}</span>
              {#if $explorerDirty}
                <span class="text-[#e2c08d]" title="Unsaved changes">●</span>
              {/if}
              {#if $explorerContent !== null && !$explorerContent.startsWith("binary file")}
                <span class="text-[#555]">{$explorerContent.split("\n").length} lines</span>
              {/if}
              <button
                class="save-btn"
                disabled={!$explorerDirty}
                title="Save (Ctrl+S)"
                onclick={() => saveExplorerFile()}>Save</button
              >
            </div>
            <div class="flex-1 min-h-0 bg-[#1e1e1e] flex flex-col">
              {#if $explorerContent === null}
                <div class="p-6 text-sm text-[#6e6e6e]">Loading…</div>
              {:else if $explorerContent.startsWith("binary file")}
                <div class="p-6 text-sm text-[#6e6e6e]">
                  Binary file <span class="font-mono">{$explorerFile}</span> — no text preview.
                </div>
              {:else if $explorerContent.endsWith("… [truncated]")}
                <div class="px-3 py-1.5 text-xs text-[#d7ba7d] bg-[#3a3320]">
                  File is large — showing truncated preview. Editing is disabled.
                </div>
                <pre class="flex-1 overflow-auto p-3 text-[12px] font-mono leading-5 text-[#9d9d9d] whitespace-pre-wrap break-all">{$explorerContent}</pre>
              {:else}
                <textarea
                  bind:value={$explorerContent}
                  oninput={() => explorerDirty.set(true)}
                  onkeydown={(e) => {
                    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
                      e.preventDefault();
                      saveExplorerFile();
                    }
                  }}
                  spellcheck="false"
                  class="flex-1 w-full bg-[#1e1e1e] text-[13px] font-mono leading-5 text-[#d4d4d4] p-3 outline-none resize-none"
                ></textarea>
              {/if}
            </div>
          {:else}
            <div class="flex-1 flex items-center justify-center text-sm text-[#6e6e6e]">
              Select a file to preview its contents
            </div>
          {/if}
        {:else if $view === "changes" || $view === "history"}
          {#if $selectedFile}
            <DiffView diff={$selectedDiff} />
          {:else}
            <div class="flex-1 flex items-center justify-center text-sm text-[#6e6e6e]">
              Select a file to view its diff
            </div>
          {/if}
        {:else}
          <div class="flex-1 flex items-center justify-center text-sm text-[#6e6e6e]">
            Use the panel to manage {$view}
          </div>
        {/if}
      </main>
      {/if}
    </div>
    <StatusBar />
    <div class="hidden">{$statusBar ? "" : ""}</div>
  {/if}
</div>

<style>
  .tb-btn {
    color: #cccccc;
    padding: 3px 8px;
    border-radius: 4px;
  }
  .tb-btn:hover {
    background: #3e3e42;
    color: #fff;
  }
  .tb-btn.primary {
    background: #0e639c;
    color: #fff;
  }
  .tb-btn.primary:hover {
    background: #1177bb;
  }
  .nav-btn {
    width: 36px;
    height: 36px;
    font-size: 17px;
    color: #858585;
    border-radius: 6px;
    border-left: 2px solid transparent;
  }
  .nav-btn:hover {
    color: #fff;
  }
  .nav-btn.active {
    color: #fff;
    border-left-color: #007fd4;
    background: #2d2d2d;
  }
  .layout-btn {
    padding: 1px 7px;
    border-radius: 4px;
    color: #858585;
    font-size: 13px;
  }
  .layout-btn:hover {
    color: #fff;
    background: #3e3e42;
  }
  .layout-btn.active {
    color: #fff;
    background: #094771;
  }
  .save-btn {
    background: #0e639c;
    color: #fff;
    font-size: 11px;
    padding: 2px 10px;
    border-radius: 3px;
  }
  .save-btn:hover:not(:disabled) {
    background: #1177bb;
  }
  .save-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
