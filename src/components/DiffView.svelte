<script lang="ts">
  import type { FileDiff } from "$lib/types";
  import { diffMode } from "$lib/store";

  let { diff }: { diff: FileDiff | null } = $props();

  function lineClass(kind: string): string {
    switch (kind) {
      case "add":
        return "bg-[#1a3319] text-[#b5cea8]";
      case "del":
        return "bg-[#3f1d1d] text-[#ce9178]";
      default:
        return "text-[#9d9d9d]";
    }
  }
</script>

<div class="flex-1 overflow-auto bg-[#1e1e1e]">
  {#if !diff}
    <div class="p-6 text-sm text-[#6e6e6e]">Loading diff…</div>
  {:else if diff.binary}
    <div class="p-6 text-sm text-[#6e6e6e]">
      Binary file <span class="font-mono">{diff.path}</span> — no text preview.
    </div>
  {:else if diff.hunks.length === 0}
    <div class="p-6 text-sm text-[#6e6e6e]">No changes to show.</div>
  {:else}
    <div class="px-3 py-2 text-xs text-[#6e6e6e] border-b border-[#2d2d2d]">
      <span class="font-mono">{diff.path}</span>
      <span class="ml-2 text-green-400">+{diff.added}</span>
      <span class="ml-1 text-red-400">−{diff.removed}</span>
      <span class="ml-3">
        <button
          class="underline"
          class:font-bold={$diffMode === "inline"}
          onclick={() => diffMode.set("inline")}>inline</button
        >
        |
        <button
          class="underline"
          class:font-bold={$diffMode === "side"}
          onclick={() => diffMode.set("side")}>side-by-side</button
        >
      </span>
    </div>
    {#if $diffMode === "inline"}
      <table class="w-full text-[12px] font-mono leading-5 border-collapse">
        <tbody>
          {#each diff.hunks as hunk}
            <tr><td colspan="3" class="hunk-header">{hunk.header}</td></tr>
            {#each hunk.lines as line}
              <tr class={lineClass(line.kind)}>
                <td class="lineno">{line.old_no ?? ""}</td>
                <td class="lineno">{line.new_no ?? ""}</td>
                <td class="px-2 whitespace-pre-wrap break-all"
                  >{line.kind === "add" ? "+" : line.kind === "del" ? "−" : " "}{line.content}</td
                >
              </tr>
            {/each}
          {/each}
        </tbody>
      </table>
    {:else}
      {#each diff.hunks as hunk}
        <div class="hunk-header">{hunk.header}</div>
        <div class="grid grid-cols-2 text-[12px] font-mono leading-5">
          <div class="border-r border-[#2d2d2d]">
            {#each hunk.lines.filter((l) => l.kind !== "add") as line}
              <div class={`px-2 whitespace-pre-wrap break-all ${lineClass(line.kind === "del" ? "del" : "context")}`}>
                <span class="text-[#555] mr-2">{line.old_no ?? ""}</span>{line.content}
              </div>
            {/each}
          </div>
          <div>
            {#each hunk.lines.filter((l) => l.kind !== "del") as line}
              <div class={`px-2 whitespace-pre-wrap break-all ${lineClass(line.kind === "add" ? "add" : "context")}`}>
                <span class="text-[#555] mr-2">{line.new_no ?? ""}</span>{line.content}
              </div>
            {/each}
          </div>
        </div>
      {/each}
    {/if}
  {/if}
</div>

<style>
  .lineno {
    width: 44px;
    min-width: 44px;
    text-align: right;
    padding: 0 8px;
    color: #6e6e6e;
    user-select: none;
    vertical-align: top;
  }
  .hunk-header {
    background: #2d2d2d;
    color: #569cd6;
    padding: 2px 12px;
  }
</style>
