<script lang="ts">
  import {
    busy,
    error,
    expandExplorerDir,
    explorerCollapse,
    explorerCollapsed,
    explorerCreate,
    explorerDelete,
    explorerEntries,
    explorerError,
    explorerExpand,
    explorerFile,
    explorerLoaded,
    explorerRename,
    openExplorerFile,
    refreshExplorer,
    reloadExplorerDir,
    status,
    view,
  } from "$lib/store";
  import type { FileEntry, TreeEntry } from "$lib/types";

  let filter = $state("");
  let creating = $state<{ dir: string; kind: "file" | "dir" } | null>(null);
  let newName = $state("");
  let renaming = $state<string | null>(null);
  let renameValue = $state("");
  let menuFor = $state<string | null>(null);

  function focusInput(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  function cancelCreate() {
    creating = null;
    newName = "";
  }

  const rootEntries = $derived($explorerEntries[""] ?? []);

  const gitBadges = $derived.by(() => {
    const map = new Map<string, string>();
    const s = $status;
    if (!s) return map;
    const put = (f: FileEntry) => {
      const b = badgeOf(f);
      if (b) map.set(f.path, b);
    };
    s.staged.forEach(put);
    s.unstaged.forEach(put);
    s.untracked.forEach(put);
    s.unmerged.forEach(put);
    return map;
  });

  function badgeOf(f: FileEntry): string {
    const st = f.index_status ?? f.worktree_status;
    switch (st) {
      case "modified":
        return "M";
      case "added":
      case "untracked":
        return "U";
      case "deleted":
        return "D";
      case "unmerged":
        return "!";
      default:
        return "";
    }
  }

  function badgeClass(b: string): string {
    switch (b) {
      case "U":
        return "text-green-400";
      case "D":
        return "text-red-400";
      case "!":
        return "text-red-500 font-bold";
      default:
        return "text-amber-400";
    }
  }

  function matches(f: TreeEntry): boolean {
    if (!filter.trim()) return true;
    return fileName(f.path).toLowerCase().includes(filter.trim().toLowerCase());
  }

  function entriesFor(dir: string): TreeEntry[] {
    return ($explorerEntries[dir] ?? []).filter(matches);
  }

  async function ensureLoaded(path: string) {
    let done = false;
    explorerLoaded.subscribe((s) => (done = s.has(path)))();
    if (!done) await expandExplorerDir(path);
  }

  // Unloaded folders render collapsed — the caret must match what's shown.
  function isCollapsed(path: string): boolean {
    return $explorerCollapsed.has(path) || !$explorerLoaded.has(path);
  }

  async function toggleDir(path: string) {
    if (!isCollapsed(path)) {
      explorerCollapse(path);
      return;
    }
    try {
      await ensureLoaded(path);
      explorerExpand(path);
    } catch {
      error.set(`Could not open folder ${path}`);
      return;
    }
  }

  async function reveal(path: string) {
    // Expand every ancestor so a filtered/selected file is visible.
    const parts = path.split("/");
    let prefix = "";
    for (let i = 0; i < parts.length - 1; i++) {
      prefix = prefix ? `${prefix}/${parts[i]}` : parts[i];
      explorerCollapsed.update((s) => {
        if (!s.has(prefix)) return s;
        const next = new Set(s);
        next.delete(prefix);
        return next;
      });
      await ensureLoaded(prefix);
    }
  }

  function depthOf(path: string): number {
    return path.split("/").length - 1;
  }

  function fileName(path: string): string {
    const i = path.lastIndexOf("/");
    return i >= 0 ? path.slice(i + 1) : path;
  }

  function parentDir(path: string): string {
    const i = path.lastIndexOf("/");
    return i >= 0 ? path.slice(0, i) : "";
  }

  function iconFor(name: string): string {
    const ext = name.includes(".") ? name.split(".").pop()!.toLowerCase() : "";
    switch (ext) {
      case "ts":
      case "tsx":
      case "js":
      case "jsx":
      case "mjs":
        return "{ }";
      case "svelte":
      case "vue":
        return "◆";
      case "rs":
        return "🦀";
      case "py":
        return "🐍";
      case "json":
      case "toml":
      case "yaml":
      case "yml":
        return "≡";
      case "md":
      case "txt":
        return "☰";
      case "css":
      case "scss":
        return "#";
      case "html":
        return "<>";
      case "png":
      case "jpg":
      case "jpeg":
      case "gif":
      case "svg":
      case "ico":
      case "icns":
        return "◫";
      case "lock":
        return "🔒";
      default:
        return "◦";
    }
  }

  async function submitCreate() {
    if (!creating || !newName.trim()) {
      creating = null;
      return;
    }
    const dir = creating.dir;
    busy.set(true);
    try {
      await explorerCreate(creating.kind, dir, newName.trim());
      explorerCollapsed.update((s) => {
        if (!s.has(dir)) return s;
        const next = new Set(s);
        next.delete(dir);
        return next;
      });
    } finally {
      busy.set(false);
      creating = null;
      newName = "";
    }
  }

  async function submitRename(path: string) {
    const target = renameValue.trim().replace(/\\/g, "/");
    renaming = null;
    if (!target || target === fileName(path)) return;
    const dir = parentDir(path);
    const dest = dir ? `${dir}/${target}` : target;
    await reveal(path);
    await explorerRename(path, dest, dir);
  }

  async function removeEntry(path: string) {
    const dir = parentDir(path);
    if (!confirm(`Delete ${path}?`)) return;
    menuFor = null;
    await explorerDelete(path, dir);
  }

  function startRename(path: string) {
    menuFor = null;
    renaming = path;
    renameValue = fileName(path);
  }

  $effect(() => {
    if ($view === "explorer" && rootEntries.length === 0) refreshExplorer();
  });
</script>

{#snippet row(f: TreeEntry, depth: number)}
  <li>
    {#if f.kind === "dir"}
      <div
        class="folder-row"
        class:menu-open={menuFor === f.path}
        style="padding-left: {12 + depth * 14}px"
        role="button"
        tabindex="0"
        title={f.path}
        onclick={() => toggleDir(f.path)}
        onkeydown={(e) => e.key === "Enter" && toggleDir(f.path)}
        oncontextmenu={(e) => {
          e.preventDefault();
          menuFor = menuFor === f.path ? null : f.path;
        }}
      >
        <span class="caret">{isCollapsed(f.path) ? "▸" : "▾"}</span>
        {#if renaming === f.path}
          <input
            class="rename-input"
            bind:value={renameValue}
            use:focusInput
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => {
              e.stopPropagation();
              if (e.key === "Enter") submitRename(f.path);
              if (e.key === "Escape") renaming = null;
            }}
          />
        {:else}
          <span class="truncate flex-1">📁 {fileName(f.path)}</span>
        {/if}
        {#if gitBadges.get(f.path)}
          <span class={`text-xs font-mono ${badgeClass(gitBadges.get(f.path)!)}`}>{gitBadges.get(f.path)}</span>
        {/if}
      </div>
      {#if menuFor === f.path}
        <div class="ctx-menu" style="margin-left: {26 + depth * 14}px">
          <button onclick={async () => { await ensureLoaded(f.path); explorerExpand(f.path); creating = { dir: f.path, kind: "file" }; newName = ""; menuFor = null; }}>＋ New file</button>
          <button onclick={async () => { await ensureLoaded(f.path); explorerExpand(f.path); creating = { dir: f.path, kind: "dir" }; newName = ""; menuFor = null; }}>＋ New folder</button>
          <button onclick={() => startRename(f.path)}>✎ Rename</button>
          <button class="danger" onclick={() => removeEntry(f.path)}>🗑 Delete</button>
          <button onclick={() => reloadExplorerDir(f.path).then(() => { menuFor = null; })}>⟳ Reload</button>
        </div>
      {/if}
      {#if creating?.dir === f.path && !isCollapsed(f.path)}
        <div class="create-row" style="padding-left: {26 + depth * 14}px">
          <span class="text-[#858585]">{creating.kind === "file" ? "◦" : "📁"}</span>
          <input
            class="rename-input"
            placeholder={creating.kind === "file" ? "filename" : "folder name"}
            bind:value={newName}
            use:focusInput
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => {
              e.stopPropagation();
              if (e.key === "Enter") submitCreate();
              if (e.key === "Escape") cancelCreate();
            }}
          />
          <button
            class="cancel-btn"
            title="Cancel"
            onclick={(e) => {
              e.stopPropagation();
              cancelCreate();
            }}>✕</button
          >
        </div>
      {/if}
      {#if !isCollapsed(f.path)}
        {#if entriesFor(f.path).length === 0}
          <div class="text-xs text-[#6e6e6e] italic" style="padding-left: {26 + depth * 14}px">empty</div>
        {:else}
          {#each entriesFor(f.path) as child (child.path)}
            {@render row(child, depth + 1)}
          {/each}
        {/if}
      {/if}
    {:else}
      <div
        class="file-row"
        class:selected={$explorerFile === f.path}
        class:menu-open={menuFor === f.path}
        style="padding-left: {12 + depth * 14}px"
        role="button"
        tabindex="0"
        title={f.path}
        onclick={() => openExplorerFile(f.path)}
        onkeydown={(e) => e.key === "Enter" && openExplorerFile(f.path)}
        oncontextmenu={(e) => {
          e.preventDefault();
          menuFor = menuFor === f.path ? null : f.path;
        }}
      >
        {#if renaming === f.path}
          <input
            class="rename-input"
            bind:value={renameValue}
            use:focusInput
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => {
              e.stopPropagation();
              if (e.key === "Enter") submitRename(f.path);
              if (e.key === "Escape") renaming = null;
            }}
          />
        {:else}
          <span class="file-icon">{iconFor(fileName(f.path))}</span>
          <span class="truncate flex-1 text-left">{fileName(f.path)}</span>
        {/if}
        {#if gitBadges.get(f.path)}
          <span class={`text-xs font-mono ${badgeClass(gitBadges.get(f.path)!)}`}>{gitBadges.get(f.path)}</span>
        {/if}
      </div>
      {#if menuFor === f.path}
        <div class="ctx-menu" style="margin-left: {26 + depth * 14}px">
          <button onclick={() => startRename(f.path)}>✎ Rename</button>
          <button class="danger" onclick={() => removeEntry(f.path)}>🗑 Delete</button>
        </div>
      {/if}
    {/if}
  </li>
{/snippet}

<div
  class="flex-1 overflow-auto"
  role="button"
  tabindex="0"
  aria-label="Explorer file list"
  onclick={() => {
    menuFor = null;
    if (creating) cancelCreate();
  }}
  onkeydown={(e) => {
    if (e.key === "Escape") menuFor = null;
  }}
>
  <div class="flex items-center gap-1 px-3 py-1">
    <span class="text-xs uppercase tracking-wide text-[#bbbbbb] font-semibold">Explorer</span>
    <span class="ml-auto flex gap-0.5">
      <button class="hdr-btn" title="New file in root" onclick={(e) => { e.stopPropagation(); creating = { dir: "", kind: "file" }; newName = ""; }}>＋🗋</button>
      <button class="hdr-btn" title="New folder in root" onclick={(e) => { e.stopPropagation(); creating = { dir: "", kind: "folder" as "dir" }; newName = ""; }}>＋📁</button>
      <button
        class="hdr-btn"
        title="Refresh file list"
        onclick={(e) => {
          e.stopPropagation();
          refreshExplorer();
        }}>⟳</button
      >
    </span>
  </div>
  <div class="px-3 pb-1">
    <input
      bind:value={filter}
      placeholder="filter files…"
      class="w-full bg-[#3c3c3c] text-[12px] rounded px-2 py-0.5 outline-none placeholder:text-[#6e6e6e]"
    />
  </div>
  {#if creating?.dir === ""}
    <div class="create-row" style="padding-left: 26px">
      <span class="text-[#858585]">{creating.kind === "file" ? "◦" : "📁"}</span>
      <input
        class="rename-input"
        placeholder={creating.kind === "file" ? "filename" : "folder name"}
        bind:value={newName}
        use:focusInput
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => {
          e.stopPropagation();
          if (e.key === "Enter") submitCreate();
          if (e.key === "Escape") cancelCreate();
        }}
      />
      <button
        class="cancel-btn"
        title="Cancel"
        onclick={(e) => {
          e.stopPropagation();
          cancelCreate();
        }}>✕</button
      >
    </div>
  {/if}
  {#if $explorerError}
    <div class="mx-3 mb-1 text-xs text-[#f48771] bg-[#5a1d1d] rounded px-2 py-1 break-words">{$explorerError}</div>
  {/if}
  {#if rootEntries.length === 0 && !$explorerError}
    <div class="px-3 py-1 text-xs text-[#6e6e6e] italic">No files.</div>
  {:else}
    <ul>
      {#each rootEntries.filter(matches) as f (f.path)}
        {@render row(f, depthOf(f.path))}
      {/each}
    </ul>
  {/if}
</div>

<style>
  .folder-row,
  .file-row {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    padding-top: 2px;
    padding-bottom: 2px;
    padding-right: 8px;
    font-size: 13px;
    cursor: pointer;
    background: transparent;
    border: none;
    text-align: left;
    color: #cccccc;
  }
  .folder-row {
    font-weight: 600;
    color: #bbbbbb;
  }
  .folder-row:hover,
  .file-row:hover {
    background: #2a2d2e;
  }
  .file-row.selected {
    background: #37373d;
  }
  .menu-open {
    background: #2a2d2e;
  }
  .caret {
    width: 12px;
    color: #858585;
    font-size: 11px;
  }
  .file-icon {
    width: 26px;
    color: #858585;
    font-size: 11px;
    text-align: center;
  }
  .rename-input {
    flex: 1;
    background: #3c3c3c;
    color: #fff;
    font-size: 13px;
    border: 1px solid #007fd4;
    border-radius: 2px;
    padding: 0 4px;
    outline: none;
    min-width: 0;
  }
  .cancel-btn {
    color: #858585;
    font-size: 12px;
    padding: 1px 5px;
    border-radius: 3px;
  }
  .cancel-btn:hover {
    background: #5a1d1d;
    color: #f14c4c;
  }
  .create-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding-right: 8px;
    padding-top: 1px;
    padding-bottom: 1px;
  }
  .ctx-menu {
    display: flex;
    flex-direction: column;
    background: #252526;
    border: 1px solid #454545;
    border-radius: 4px;
    margin: 2px 8px 2px 0;
    padding: 3px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
  }
  .ctx-menu button {
    text-align: left;
    font-size: 12px;
    color: #cccccc;
    padding: 3px 8px;
    border-radius: 3px;
  }
  .ctx-menu button:hover {
    background: #094771;
    color: #fff;
  }
  .ctx-menu button.danger:hover {
    background: #5a1d1d;
    color: #f14c4c;
  }
  .hdr-btn {
    padding: 1px 5px;
    border-radius: 4px;
    color: #858585;
    font-size: 12px;
  }
  .hdr-btn:hover {
    color: #fff;
    background: #3e3e42;
  }
</style>
