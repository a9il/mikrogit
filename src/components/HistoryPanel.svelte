<script lang="ts">
  import { git } from "$lib/gitClient";
  import {
    commits,
    refreshLog,
    runOp,
    selectedCommit,
    selectedDiff,
    selectedFile,
    view,
  } from "$lib/store";

  let commitFiles = $state<string[]>([]);
  let expanded = $state<string | null>(null);

  $effect(() => {
    if ($view === "history" && $commits.length === 0) refreshLog();
  });

  async function select(hash: string) {
    selectedCommit.set(hash);
    expanded = hash;
    commitFiles = await git.getCommitFiles(hash).catch(() => []);
    selectedFile.set(null);
    selectedDiff.set(null);
    if (commitFiles.length > 0) {
      selectedFile.set({ path: commitFiles[0], staged: false });
      selectedDiff.set(await git.getCommitDiff(hash, commitFiles[0]).catch(() => null));
    }
  }

  async function showFile(hash: string, path: string) {
    selectedFile.set({ path, staged: false });
    selectedDiff.set(await git.getCommitDiff(hash, path).catch(() => null));
  }
</script>

<div class="flex-1 overflow-auto">
  {#each $commits as c (c.hash)}
    <div>
      <button
        class="commit-row"
        class:selected={$selectedCommit === c.hash}
        onclick={() => select(c.hash)}
      >
        <div class="truncate text-[13px] text-white">{c.subject}</div>
        <div class="text-xs text-[#6e6e6e]">
          {c.author} · {c.short_hash} · {c.date.slice(0, 16).replace("T", " ")}
          {#if c.refs.length > 0}
            <span class="text-[#4ec9b0]"> [{c.refs.join(", ")}]</span>
          {/if}
        </div>
      </button>
      {#if expanded === c.hash && commitFiles.length > 0}
        <div class="ml-4 border-l border-[#2d2d2d] pl-2 pb-2">
          {#each commitFiles as f}
            <button
              class="block w-full text-left text-xs font-mono text-[#999] hover:text-white truncate px-2 py-0.5"
              onclick={() => showFile(c.hash, f)}>{f}</button
            >
          {/each}
        </div>
      {/if}
    </div>
  {:else}
    <div class="p-4 text-sm text-[#6e6e6e]">No commits yet.</div>
  {/each}
  <div class="p-2">
    <button
      class="text-xs text-[#999] hover:text-white underline"
      onclick={() => runOp(async () => { commits.set(await git.getLog(200, $commits.length)); return null; })}
    >Load more</button>
  </div>
</div>

<style>
  .commit-row {
    display: block;
    width: 100%;
    text-align: left;
    padding: 5px 12px;
    border: none;
    background: transparent;
    cursor: pointer;
  }
  .commit-row:hover {
    background: #2a2d2e;
  }
  .commit-row.selected {
    background: #37373d;
  }
</style>
