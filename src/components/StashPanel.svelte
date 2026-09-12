<script lang="ts">
  import { git } from "$lib/gitClient";
  import { refreshStashes, runOp, stashes, view } from "$lib/store";

  let newMsg = $state("");
  let includeUntracked = $state(false);

  $effect(() => {
    if ($view === "stashes") refreshStashes();
  });

  async function create() {
    await runOp(() => git.stashCreate(newMsg || undefined, includeUntracked));
    newMsg = "";
    await refreshStashes();
  }
</script>

<div class="flex-1 overflow-auto p-2">
  <div class="px-2 py-1 text-xs uppercase tracking-wide text-[#bbbbbb] font-semibold">Stash</div>
  <div class="px-2 py-1 flex flex-col gap-1">
    <input
      bind:value={newMsg}
      placeholder="stash message (optional)"
      class="bg-[#3c3c3c] text-[13px] rounded px-2 py-1 outline-none"
    />
    <div class="flex items-center gap-2">
      <label class="text-xs text-[#999] flex items-center gap-1">
        <input type="checkbox" bind:checked={includeUntracked} /> include untracked
      </label>
      <button class="ml-auto bg-[#0e639c] text-white text-xs px-3 py-1 rounded" onclick={create}>
        Stash changes
      </button>
    </div>
  </div>
  {#each $stashes as s (s.index)}
    <div class="stash-row">
      <span class="flex-1 truncate text-[13px]">
        <span class="font-mono text-[#569cd6]">stash@{s.index}</span>
        <span class="ml-2">{s.message}</span>
      </span>
      <button class="mini-btn" onclick={() => runOp(() => git.stashApply(s.index)).then(refreshStashes)}>apply</button>
      <button class="mini-btn" onclick={() => runOp(() => git.stashPop(s.index)).then(refreshStashes)}>pop</button>
      <button class="mini-btn danger" onclick={() => runOp(() => git.stashDrop(s.index)).then(refreshStashes)}>drop</button>
    </div>
  {:else}
    <div class="p-3 text-xs text-[#6e6e6e]">No stashes.</div>
  {/each}
</div>

<style>
  .stash-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-radius: 4px;
  }
  .stash-row:hover {
    background: #2a2d2e;
  }
  .mini-btn {
    font-size: 11px;
    color: #999;
    padding: 1px 6px;
    border-radius: 3px;
    white-space: nowrap;
  }
  .mini-btn:hover {
    background: #3e3e42;
    color: #fff;
  }
  .mini-btn.danger:hover {
    background: #5a1d1d;
    color: #f14c4c;
  }
</style>
