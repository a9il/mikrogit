<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { git } from "$lib/gitClient";
  import { currentRepo, initConsole, initWatcher, openRepo, refresh } from "$lib/store";
  import SourceControl from "../components/SourceControl.svelte";

  onMount(async () => {
    initWatcher();
    initConsole();
    try {
      const saved = localStorage.getItem("mikrogit.repo");
      const current = await git.currentRepo().catch(() => null);
      const repo = current ?? saved;
      if (repo) {
        await openRepo(repo);
      }
    } catch {
      // No repo yet — show welcome screen.
    }
    currentRepo.subscribe((p) => {
      if (p) localStorage.setItem("mikrogit.repo", p);
    });
  });
</script>

<SourceControl />

{#if false}{refresh}{/if}
