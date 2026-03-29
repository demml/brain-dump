<script lang="ts">
  import { onMount } from "svelte";
  import { listNodes } from "$lib/tauri";
  import type { Node } from "$lib/types";

  let projects = $state<Node[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      projects = await listNodes("project");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });
</script>

<div class="max-w-6xl mx-auto">
  <div class="flex items-center justify-between mb-8">
    <h1 class="text-2xl font-bold text-[var(--color-phosphor-bright)]">brain-dump</h1>
    <button
      class="px-4 py-2 border-2 border-[var(--color-border-bright)] bg-[var(--color-bg-card)] text-[var(--color-phosphor)] hover:bg-[var(--color-bg-card-hover)] transition-colors"
      style="box-shadow: var(--shadow-hard);"
    >
      + New Project
    </button>
  </div>

  {#if loading}
    <p class="text-[var(--color-phosphor-dim)]">Loading...</p>
  {:else if error}
    <div class="border-2 border-[var(--color-danger)] p-4 text-[var(--color-danger)]">
      <p>Failed to load projects: {error}</p>
    </div>
  {:else if projects.length === 0}
    <div class="border-2 border-dashed border-[var(--color-border)] p-8 text-center text-[var(--color-phosphor-dim)]">
      <p>No projects yet. Create one to get started.</p>
      <p class="text-sm mt-2">Or run: <code class="text-[var(--color-phosphor)]">bd new project "My Idea"</code></p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each projects as project}
        <a
          href="/project/{project.id}"
          class="block p-4 border-2 border-[var(--color-border)] bg-[var(--color-bg-card)] hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-none transition-all"
          style="box-shadow: var(--shadow-hard);"
        >
          <h3 class="font-bold text-[var(--color-phosphor-bright)] mb-2">{project.title}</h3>
          <p class="text-sm text-[var(--color-phosphor-dim)] line-clamp-2">{(project.description ?? "").slice(0, 100)}</p>
          <div class="mt-3 flex items-center gap-2 text-xs text-[var(--color-phosphor-muted)]">
            <span>{project.status}</span>
          </div>
        </a>
      {/each}
    </div>
  {/if}
</div>
