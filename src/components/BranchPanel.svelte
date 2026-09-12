<script lang="ts">
  import { git } from "$lib/gitClient";
  import {
    branches,
    error,
    refreshBranches,
    runOp,
    view,
  } from "$lib/store";

  let newName = $state("");
  let showCreate = $state(false);

  $effect(() => {
    if ($view === "branches") refreshBranches();
  });

  async function checkout(name: string) {
    await runOp(() => git.checkout(name));
    await refreshBranches();
  }

  async function create() {
    if (!newName.trim()) return;
    const ok = await runOp(() => git.createBranch(newName.trim(), true));
    if (ok !== null) {
      newName = "";
      showCreate = false;
      await refreshBranches();
    }
  }

  async function remove(name: string) {
    if (!confirm(`Delete branch "${name}"?`)) return;
    const ok = await runOp(() => git.deleteBranch(name, false));
    if (ok === null && $error?.includes("not fully merged")) {
      if (confirm(`"${name}" is not fully merged. Force delete?`)) {
        await runOp(() => git.deleteBranch(name, true));
      }
    }
    await refreshBranches();
  }
</script>

<div class="flex-1 overflow-auto p-2">
  <div class="flex items-center px-2 py-1">
    <span class="text-xs uppercase tracking-wide text-[#bbbbbb] font-semibold">Branches</span>
    <button class="ml-auto text-xs text-[#999] hover:text-white" onclick={() => (showCreate = !showCreate)}>
      + New
    </button>
  </div>
  {#if showCreate}
    <div class="flex gap-1 px-2 py-1">
      <input
        bind:value={newName}
        placeholder="branch name"
        class="flex-1 bg-[#3c3c3c] text-[13px] rounded px-2 py-1 outline-none"
        onkeydown={(e) => e.key === "Enter" && create()}
      />
      <button class="bg-[#0e639c] text-white text-xs px-3 rounded" onclick={create}>Create</button>
    </div>
  {/if}
  {#each $branches.filter((b) => !b.remote) as b (b.name)}
    <div class="branch-row" class:current={b.current}>
      <span class="flex-1 truncate font-mono text-[13px]">
        {#if b.current}<span class="text-green-400">● </span>{/if}{b.name}
      </span>
      {#if b.ahead > 0}<span class="text-xs text-[#999]">↑{b.ahead}</span>{/if}
      {#if b.behind > 0}<span class="text-xs text-[#999]">↓{b.behind}</span>{/if}
      {#if !b.current}
        <button class="mini-btn" title="Checkout" onclick={() => checkout(b.name)}>checkout</button>
        <button class="mini-btn danger" title="Delete" onclick={() => remove(b.name)}>✕</button>
      {/if}
    </div>
    {#if b.last_commit_subject}
      <div class="px-4 text-xs text-[#6e6e6e] truncate">{b.last_commit} {b.last_commit_subject}</div>
    {/if}
  {/each}
  {#if $branches.some((b) => b.remote)}
    <div class="px-2 pt-3 text-xs uppercase tracking-wide text-[#bbbbbb] font-semibold">Remotes</div>
    {#each $branches.filter((b) => b.remote) as b (b.name)}
      <div class="branch-row text-[#999]">
        <span class="flex-1 truncate font-mono text-[13px]">{b.name}</span>
        <button class="mini-btn" title="Checkout" onclick={() => checkout(b.name)}>checkout</button>
      </div>
    {/each}
  {/if}
</div>

<style>
  .branch-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    border-radius: 4px;
  }
  .branch-row:hover {
    background: #2a2d2e;
  }
  .branch-row.current {
    background: #37373d;
  }
  .mini-btn {
    font-size: 11px;
    color: #999;
    padding: 1px 6px;
    border-radius: 3px;
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
