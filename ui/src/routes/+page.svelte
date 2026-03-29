<script lang="ts">
  import { onMount } from "svelte";
  import { listNodes } from "$lib/tauri";
  import type { Node } from "$lib/types";
  import ProjectCard from "$lib/components/ProjectCard.svelte";
  import NodeForm from "$lib/components/NodeForm.svelte";

  let projects = $state<Node[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showNewProject = $state(false);

  onMount(async () => {
    try {
      projects = await listNodes("project");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function onProjectSaved(_node: Node) {
    showNewProject = false;
    try {
      projects = await listNodes("project");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<div class="max-w-6xl mx-auto">
  <div class="flex items-center justify-between mb-8">
    <h1 class="text-2xl font-bold" style="color: var(--color-phosphor-bright);">brain-dump</h1>
    <button
      class="px-4 py-2 border-2 transition-colors"
      style="
        border-color: var(--color-border-bright);
        background: var(--color-bg-card);
        color: var(--color-phosphor);
        box-shadow: var(--shadow-hard);
      "
      onclick={() => showNewProject = true}
    >
      + New Project
    </button>
  </div>

  {#if loading}
    <p style="color: var(--color-phosphor-dim);">Loading...</p>
  {:else if error}
    <div class="border-2 p-4" style="border-color: var(--color-danger); color: var(--color-danger);">
      <p>Failed to load projects: {error}</p>
    </div>
  {:else if projects.length === 0}
    <div class="border-2 border-dashed p-8 text-center" style="border-color: var(--color-border); color: var(--color-phosphor-dim);">
      <p>No projects yet. Create one to get started.</p>
      <p class="text-sm mt-2">Or run: <code style="color: var(--color-phosphor);">bd new project "My Idea"</code></p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each projects as project}
        <ProjectCard {project} />
      {/each}
    </div>
  {/if}
</div>

{#if showNewProject}
  <NodeForm
    nodeType="project"
    onSave={onProjectSaved}
    onCancel={() => showNewProject = false}
  />
{/if}
