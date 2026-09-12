<script lang="ts">
  import { git } from "$lib/gitClient";
  import { commitAmend, commitMessage, refresh, runOp, status } from "$lib/store";

  let output = $state("");

  async function doCommit() {
    const message = $commitMessage.trim();
    if (!message) return;
    const hash = await runOp(() => git.commit(message, $commitAmend));
    if (hash) {
      output = `Committed ${(hash as string).slice(0, 7)}`;
      commitMessage.set("");
      commitAmend.set(false);
    }
  }
</script>

<div class="border-b border-[#2d2d2d] p-3">
  <textarea
    bind:value={$commitMessage}
    placeholder="Message (Ctrl+Enter to commit)"
    rows="3"
    data-testid="commit-message"
    class="w-full bg-[#3c3c3c] text-[13px] text-white rounded p-2 outline-none focus:ring-1 focus:ring-[#007fd4] resize-y"
    onkeydown={(e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") doCommit();
    }}
  ></textarea>
  <div class="flex items-center gap-2 mt-2">
    <button
      onclick={doCommit}
      data-testid="commit-button"
      disabled={!$commitMessage.trim() || ($status && $status.staged.length === 0 && !$commitAmend)}
      class="bg-[#0e639c] hover:bg-[#1177bb] disabled:opacity-40 disabled:cursor-not-allowed text-white text-[13px] px-4 py-1 rounded"
      title="Commit staged changes"
    >
      ✓ Commit
    </button>
    <label class="text-xs text-[#999] flex items-center gap-1 cursor-pointer">
      <input type="checkbox" bind:checked={$commitAmend} /> Amend
    </label>
    {#if output}
      <span class="text-xs text-green-400">{output}</span>
    {/if}
    <button
      class="ml-auto text-xs text-[#999] hover:text-white underline"
      onclick={() => refresh()}>Refresh</button
    >
  </div>
</div>
