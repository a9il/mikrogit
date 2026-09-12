<script lang="ts">
  import { statusBar } from "$lib/store";

  const branch = $derived($statusBar?.branch ?? "(no branch)");
  const ahead = $derived($statusBar?.ahead ?? 0);
  const behind = $derived($statusBar?.behind ?? 0);
  const staged = $derived($statusBar?.staged_count ?? 0);
  const unstaged = $derived($statusBar?.unstaged_count ?? 0);
  const untracked = $derived($statusBar?.untracked_count ?? 0);
</script>

<footer class="flex items-center gap-3 px-3 py-0.5 bg-[#007acc] text-white text-xs shrink-0">
  <span class="font-mono" data-testid="status-branch">⑂ {branch}</span>
  {#if ahead > 0 || behind > 0}
    <span>↑{ahead} ↓{behind}</span>
  {/if}
  <span class="ml-auto flex gap-3" data-testid="status-counts">
    {#if staged > 0}<span>staged {staged}</span>{/if}
    {#if unstaged > 0}<span>modified {unstaged}</span>{/if}
    {#if untracked > 0}<span>untracked {untracked}</span>{/if}
    {#if staged + unstaged + untracked === 0}<span>✓ clean</span>{/if}
  </span>
</footer>
