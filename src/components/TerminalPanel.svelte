<script lang="ts">
  import { consoleEntries, runConsoleCommand, view } from "$lib/store";

  let input = $state("");
  let history = $state<string[]>([]);
  let histIdx = $state(-1);
  let outputEl: HTMLDivElement | undefined = $state();
  let inputEl: HTMLInputElement | undefined = $state();

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
  }

  $effect(() => {
    // Track entry count so the panel autoscrolls when new output lands.
    void $consoleEntries.length;
    if (outputEl) outputEl.scrollTop = outputEl.scrollHeight;
  });

  async function submit() {
    const cmdline = input.trim();
    if (!cmdline) return;
    input = "";
    history = [...history, cmdline];
    histIdx = -1;
    await runConsoleCommand(cmdline);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      void submit();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (history.length > 0) {
        histIdx = histIdx === -1 ? history.length - 1 : Math.max(0, histIdx - 1);
        input = history[histIdx];
      }
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (histIdx >= 0) {
        histIdx += 1;
        if (histIdx >= history.length) {
          histIdx = -1;
          input = "";
        } else {
          input = history[histIdx];
        }
      }
    } else if (e.key === "Escape") {
      view.set("changes");
    }
  }
</script>

<div class="terminal-panel">
  <div class="term-header">
    <span class="text-xs uppercase tracking-wide text-[#bbbbbb] font-semibold">Terminal</span>
    <span class="text-[10px] text-[#6e6e6e] ml-2">git commands — output of app actions is logged too</span>
    <span class="ml-auto flex gap-0.5">
      <button class="term-btn" title="Clear" onclick={() => consoleEntries.set([])}>⌫</button>
      <button class="term-btn" title="Close (Esc)" onclick={() => view.set("changes")}>✕</button>
    </span>
  </div>
  <div class="term-output" bind:this={outputEl} data-testid="terminal-output">
    {#if $consoleEntries.length === 0}
      <div class="text-[#6e6e6e] text-xs px-2 py-1">
        Type a git command, e.g. <span class="text-[#569cd6]">log --oneline -10</span> or
        <span class="text-[#569cd6]">add .</span>
      </div>
    {/if}
    {#each $consoleEntries as e (e.ts + e.cmd)}
      <div class="cmd-line">$ {e.cmd}</div>
      {#if e.stdout}
        <pre class="out">{e.stdout}</pre>
      {/if}
      {#if e.stderr}
        <pre class="err">{e.stderr}</pre>
      {/if}
      {#if e.code !== null && e.code !== 0}
        <div class="exit">exit code {e.code}</div>
      {/if}
    {/each}
  </div>
  <div class="term-input-row">
    <span class="prompt">git&gt;</span>
    <input
      bind:this={inputEl}
      bind:value={input}
      use:focusOnMount
      class="term-input"
      data-testid="terminal-input"
      placeholder="git command…"
      autocomplete="off"
      spellcheck="false"
      onkeydown={onKey}
    />
  </div>
</div>

<style>
  .terminal-panel {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
    border-top: 1px solid #000;
    background: #1e1e1e;
  }
  .term-header {
    display: flex;
    align-items: center;
    padding: 3px 10px;
    background: #252526;
    border-bottom: 1px solid #2d2d2d;
  }
  .term-btn {
    color: #858585;
    font-size: 12px;
    padding: 1px 6px;
    border-radius: 3px;
  }
  .term-btn:hover {
    background: #3e3e42;
    color: #fff;
  }
  .term-output {
    flex: 1;
    overflow-y: auto;
    padding: 4px 10px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 12px;
    line-height: 1.45;
  }
  .cmd-line {
    color: #4ec9b0;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .out,
  .err {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .out {
    color: #cccccc;
  }
  .err {
    color: #f48771;
  }
  .exit {
    color: #f14c4c;
    font-style: italic;
  }
  .term-input-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-top: 1px solid #2d2d2d;
  }
  .prompt {
    color: #4ec9b0;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 12px;
  }
  .term-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: #d4d4d4;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 12px;
  }
</style>
