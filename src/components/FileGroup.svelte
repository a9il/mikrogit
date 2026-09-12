<script lang="ts">
  import type { FileEntry } from "$lib/types";
  import { changesGroupsCollapsed, toggleChangesGroup } from "$lib/store";

  let {
    title,
    files,
    emptyHint,
    icons,
    onSelect,
    onStage,
    onUnstage,
    onDiscard,
    onStageAll,
    onUnstageAll,
    selectedPath,
  }: {
    title: string;
    files: FileEntry[];
    emptyHint: string;
    icons: { stage: string; discard: string };
    onSelect: (f: FileEntry) => void;
    onStage?: (f: FileEntry) => void;
    onUnstage?: (f: FileEntry) => void;
    onDiscard: (f: FileEntry) => void;
    onStageAll?: () => void;
    onUnstageAll?: () => void;
    selectedPath: string | null;
  } = $props();

  function badge(f: FileEntry): string {
    const s = f.index_status ?? f.worktree_status;
    switch (s) {
      case "modified":
        return "M";
      case "added":
        return "A";
      case "deleted":
        return "D";
      case "renamed":
        return "R";
      case "untracked":
        return "U";
      case "unmerged":
        return "!";
      default:
        return "?";
    }
  }

  function badgeClass(f: FileEntry): string {
    const s = f.index_status ?? f.worktree_status;
    switch (s) {
      case "added":
      case "untracked":
        return "text-green-400";
      case "deleted":
        return "text-red-400";
      case "unmerged":
        return "text-red-500 font-bold";
      default:
        return "text-amber-400";
    }
  }
</script>

<div class="group-section group">
  <div
    class="group-header"
    role="button"
    tabindex="0"
    onclick={() => files.length > 0 && toggleChangesGroup(title)}
    onkeydown={(e) => {
      if ((e.key === "Enter" || e.key === " ") && files.length > 0) {
        e.preventDefault();
        toggleChangesGroup(title);
      }
    }}
  >
    <span class="caret">{files.length > 0 && $changesGroupsCollapsed.has(title) ? "▸" : "▾"}</span>
    <span class="font-semibold text-xs uppercase tracking-wide text-[#bbbbbb]"
      >{title} ({files.length})</span
    >
    <span class="ml-auto flex gap-1 opacity-0 group-hover:opacity-100">
      {#if onStageAll}
        <button
          class="icon-btn"
          title="Stage all"
          onclick={(e) => {
            e.stopPropagation();
            onStageAll();
          }}>+</button
        >
      {/if}
      {#if onUnstageAll}
        <button
          class="icon-btn"
          title="Unstage all"
          onclick={(e) => {
            e.stopPropagation();
            onUnstageAll();
          }}>−</button
        >
      {/if}
    </span>
  </div>
  {#if files.length === 0}
    <div class="px-3 py-1 text-xs text-[#6e6e6e] italic">{emptyHint}</div>
  {:else if !$changesGroupsCollapsed.has(title)}
    <ul>
      {#each files as f (f.path)}
        <li>
          <button
            class="file-row"
            class:selected={selectedPath === f.path}
            onclick={() => onSelect(f)}
            title={f.original_path ? `${f.original_path} → ${f.path}` : f.path}
          >
            <span class={badgeClass(f)} style="width:14px">{badge(f)}</span>
            <span class="truncate flex-1 text-left">
              {#if f.original_path}
                <span class="text-[#6e6e6e]">{f.original_path} → </span>{f.path}
              {:else}
                {f.path}
              {/if}
            </span>
            <span class="row-actions">
              {#if onStage}
                <span
                  role="button"
                  tabindex="0"
                  class="icon-btn"
                  title={icons.stage}
                  onclick={(e) => {
                    e.stopPropagation();
                    onStage(f);
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Enter") {
                      e.stopPropagation();
                      onStage(f);
                    }
                  }}>+</span
                >
              {/if}
              {#if onUnstage}
                <span
                  role="button"
                  tabindex="0"
                  class="icon-btn"
                  title={icons.stage}
                  onclick={(e) => {
                    e.stopPropagation();
                    onUnstage(f);
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Enter") {
                      e.stopPropagation();
                      onUnstage(f);
                    }
                  }}>−</span
                >
              {/if}
              <span
                role="button"
                tabindex="0"
                class="icon-btn"
                title={icons.discard}
                onclick={(e) => {
                  e.stopPropagation();
                  onDiscard(f);
                }}
                onkeydown={(e) => {
                  if (e.key === "Enter") {
                    e.stopPropagation();
                    onDiscard(f);
                  }
                }}>↩</span
              >
            </span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .group-section {
    margin-bottom: 4px;
  }
  .group-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px 2px;
    cursor: pointer;
  }
  .caret {
    width: 12px;
    color: #858585;
    font-size: 11px;
  }
  .group-header:hover .icon-btn {
    opacity: 1;
  }
  .file-row {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 2px 12px;
    font-size: 13px;
    color: #cccccc;
    cursor: pointer;
    background: transparent;
    border: none;
    text-align: left;
  }
  .file-row:hover {
    background: #2a2d2e;
  }
  .file-row.selected {
    background: #37373d;
  }
  .file-row .row-actions {
    display: none;
    gap: 2px;
  }
  .file-row:hover .row-actions {
    display: flex;
  }
  .icon-btn {
    cursor: pointer;
    padding: 0 5px;
    border-radius: 4px;
    color: #cccccc;
    font-size: 13px;
  }
  .icon-btn:hover {
    background: #3e3e42;
  }
</style>
