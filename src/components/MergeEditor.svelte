<script lang="ts">
  import {
    conflictChoices,
    conflictFile,
    conflictPath,
    clearConflictChoice,
    loadConflicts,
    setConflictChoice,
    saveResolvedConflicts,
  } from "$lib/store";
  import { isResolved } from "$lib/mergeResolve";
  import type { ConflictRegion } from "$lib/types";

  let editing = $state<number | null>(null);
  let editBuffer = $state("");

  function startEdit(idx: number, region: ConflictRegion) {
    editing = idx;
    editBuffer = region.current_lines.join("\n") + "\n=======\n" + region.incoming_lines.join("\n");
  }

  function commitEdit(idx: number) {
    // Custom edit: replace the region's content by editing current side only.
    // We encode a custom choice as "edit:<content>".
    const text = editBuffer.replace(/\n=======\n/, "\n<<<SPLIT>>>");
    setConflictChoice(idx, `edit:${text}`);
    editing = null;
  }

  function choose(idx: number, choice: string) {
    setConflictChoice(idx, choice);
    editing = null;
  }

  function regionChoice(idx: number): string {
    return $conflictChoices[idx] ?? "";
  }

  const resolvedCount = $derived(Object.values($conflictChoices).filter(isResolved).length);
  const total = $derived($conflictFile?.regions.length ?? 0);
  const allDone = $derived(total > 0 && resolvedCount === total);
</script>

<div class="flex-1 flex flex-col overflow-hidden">
  <div class="flex items-center gap-2 px-3 py-2 border-b border-[#2d2d2d] bg-[#252526]">
    <span class="text-xs text-[#bbbbbb] font-semibold uppercase tracking-wide">Merge editor</span>
    <span class="text-xs font-mono text-[#999] truncate flex-1">{$conflictPath}</span>
    <span class="text-xs text-[#999]">{resolvedCount}/{total} resolved</span>
    <button
      class="bg-[#0e639c] hover:bg-[#1177bb] disabled:opacity-40 text-white text-xs px-3 py-1 rounded"
      disabled={!allDone}
      onclick={() => saveResolvedConflicts()}
    >
      ✓ Save &amp; stage
    </button>
  </div>

  <div class="flex-1 overflow-auto bg-[#1e1e1e]">
    {#if !$conflictFile}
      <div class="p-6 text-sm text-[#6e6e6e]">Loading conflicts…</div>
    {:else if !$conflictFile.has_conflicts}
      <div class="p-6 text-sm text-[#6e6e6e]">No conflict markers found in this file.</div>
      <div class="px-3">
        <button class="text-xs text-[#999] hover:text-white underline" onclick={() => $conflictPath && loadConflicts($conflictPath)}
          >Re-scan</button
        >
      </div>
    {:else}
      <div class="divide-y divide-[#2d2d2d]">
        {#each $conflictFile.regions as region, idx (idx)}
          {@const choice = regionChoice(idx)}
          {@const isDone = isResolved(choice)}
          <div class="border-l-2" class:border-green-500={isDone} class:border-transparent={!isDone}>
            <div class="flex items-center gap-2 px-3 py-1.5 bg-[#252526] sticky top-0">
              <span class="text-xs text-[#569cd6]">Conflict {idx + 1}</span>
              <div class="ml-auto flex gap-1">
                <button
                  class="mini-btn"
                  class:active={choice === "current"}
                  title="Accept current change"
                  onclick={() => choose(idx, "current")}>Current</button
                >
                <button
                  class="mini-btn"
                  class:active={choice === "incoming"}
                  title="Accept incoming change"
                  onclick={() => choose(idx, "incoming")}>Incoming</button
                >
                <button
                  class="mini-btn"
                  class:active={choice === "both"}
                  title="Accept both"
                  onclick={() => choose(idx, "both")}>Both</button
                >
                <button
                  class="mini-btn"
                  class:active={editing === idx}
                  title="Edit inline"
                  onclick={() => startEdit(idx, region)}>Edit</button
                >
                {#if isDone}
                  <button
                    class="mini-btn"
                    title="Undo choice"
                    onclick={() => clearConflictChoice(idx)}>↩</button
                  >
                {/if}
              </div>
            </div>

            {#if editing === idx}
              <div class="p-2">
                <textarea
                  bind:value={editBuffer}
                  rows={Math.max(4, editBuffer.split("\n").length + 1)}
                  class="w-full bg-[#1e1e1e] text-[12px] font-mono text-[#9d9d9d] p-2 outline-none border border-[#007fd4] resize-y"
                ></textarea>
                <div class="flex gap-2 mt-1">
                  <button class="text-xs text-[#999] hover:text-white underline" onclick={() => commitEdit(idx)}>Use this</button>
                  <button class="text-xs text-[#999] hover:text-white underline" onclick={() => (editing = null)}>Cancel</button>
                </div>
              </div>
            {:else}
              <div class="grid grid-cols-2 text-[12px] font-mono leading-5">
                <div class="border-r border-[#2d2d2d]">
                  <div class="px-2 py-0.5 bg-[#3f1d1d] text-[#ce9178] sticky top-[30px]">
                    ▾ {region.current_header}
                  </div>
                  {#each region.current_lines as line}
                    <div class="px-2 whitespace-pre-wrap break-all text-[#9d9d9d]">{line || " "}</div>
                  {/each}
                </div>
                <div>
                  <div class="px-2 py-0.5 bg-[#1a3319] text-[#b5cea8] sticky top-[30px]">
                    ▾ {region.incoming_header}
                  </div>
                  {#each region.incoming_lines as line}
                    <div class="px-2 whitespace-pre-wrap break-all text-[#9d9d9d]">{line || " "}</div>
                  {/each}
                </div>
              </div>
              {#if isDone}
                <div class="px-2 py-1 text-xs text-green-400 bg-[#1a3319] border-t border-[#2d2d2d]">
                  ✓ {choice === "both" ? "Keeping both" : `Keeping ${choice}`}
                </div>
              {/if}
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .mini-btn {
    font-size: 11px;
    color: #999;
    padding: 1px 7px;
    border-radius: 3px;
    border: 1px solid transparent;
  }
  .mini-btn:hover {
    background: #3e3e42;
    color: #fff;
  }
  .mini-btn.active {
    background: #094771;
    color: #fff;
    border-color: #007fd4;
  }
</style>
