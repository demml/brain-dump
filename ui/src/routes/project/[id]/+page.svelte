<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { getNode, updateNode } from "$lib/tauri";
  import type { NodeDetail } from "$lib/types";
  import MarkdownRenderer from "$lib/components/MarkdownRenderer.svelte";
  import PhaseCard from "$lib/components/PhaseCard.svelte";
  import TagBadge from "$lib/components/TagBadge.svelte";
  import ConnectionBadge from "$lib/components/ConnectionBadge.svelte";

  const id = $page.params.id;

  let detail = $state<NodeDetail | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let activeTab = $state<"phases" | "connections">("phases");

  onMount(async () => {
    try {
      detail = await getNode(id);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function archive() {
    if (!detail) return;
    try {
      await updateNode(detail.node.id, undefined, undefined, "archived");
      detail = await getNode(id);
    } catch (e) {
      console.error("archive failed:", e);
    }
  }
</script>

{#if loading}
  <div class="p-8" style="color: var(--color-phosphor-dim);">Loading…</div>
{:else if error}
  <div class="p-8 border-2" style="border-color: var(--color-danger); color: var(--color-danger);">
    Failed to load: {error}
  </div>
{:else if detail}
  <!-- Header -->
  <div class="flex items-center justify-between mb-6 pb-4 border-b-2" style="border-color: var(--color-border);">
    <div class="flex items-center gap-3">
      <a href="/" class="text-sm" style="color: var(--color-phosphor-dim);">← Back</a>
      <h1 class="text-xl font-bold" style="color: var(--color-phosphor-bright);">{detail.node.title}</h1>
      <span
        class="text-xs px-1.5 py-0.5 border"
        style="border-color: var(--color-border); color: var(--color-phosphor-dim);"
      >
        {detail.node.status}
      </span>
    </div>
    <div class="flex gap-3 text-sm" style="color: var(--color-phosphor-dim);">
      {#if detail.node.status !== "archived"}
        <button onclick={archive} style="color: var(--color-phosphor-muted);">Archive</button>
      {/if}
    </div>
  </div>

  <!-- Tags -->
  {#if detail.tags.length > 0}
    <div class="flex flex-wrap gap-2 mb-6">
      {#each detail.tags as tag}
        <TagBadge {tag} />
      {/each}
    </div>
  {/if}

  <!-- Split panel -->
  <div class="flex gap-6 min-h-0 flex-1">
    <!-- Left: markdown overview (40%) -->
    <div
      class="w-2/5 border-2 p-4 overflow-auto"
      style="border-color: var(--color-border); background: var(--color-bg-card); max-height: calc(100vh - 200px);"
    >
      <h2 class="text-sm font-bold mb-3 pb-2 border-b" style="color: var(--color-phosphor-dim); border-color: var(--color-border);">Overview</h2>
      <MarkdownRenderer content={detail.node.description} />
    </div>

    <!-- Right: tabbed content (60%) -->
    <div class="flex-1 flex flex-col min-w-0">
      <!-- Tabs -->
      <div class="flex gap-4 mb-4 border-b" style="border-color: var(--color-border);">
        <button
          class="pb-2 text-sm font-bold transition-colors"
          style="
            color: {activeTab === 'phases' ? 'var(--color-phosphor)' : 'var(--color-phosphor-muted)'};
            border-bottom: {activeTab === 'phases' ? '2px solid var(--color-phosphor)' : '2px solid transparent'};
          "
          onclick={() => activeTab = "phases"}
        >
          Phases & Tasks
        </button>
        <button
          class="pb-2 text-sm font-bold transition-colors"
          style="
            color: {activeTab === 'connections' ? 'var(--color-phosphor)' : 'var(--color-phosphor-muted)'};
            border-bottom: {activeTab === 'connections' ? '2px solid var(--color-phosphor)' : '2px solid transparent'};
          "
          onclick={() => activeTab = "connections"}
        >
          Connections
          {#if detail.blocks.length + detail.blocked_by.length + detail.related.length > 0}
            <span class="ml-1 text-xs" style="color: var(--color-phosphor-dim);">
              ({detail.blocks.length + detail.blocked_by.length + detail.related.length})
            </span>
          {/if}
        </button>
      </div>

      <!-- Tab content -->
      <div class="flex-1 overflow-auto" style="max-height: calc(100vh - 260px);">
        {#if activeTab === "phases"}
          {#if detail.children.length === 0}
            <div class="border-2 border-dashed p-6 text-center" style="border-color: var(--color-border); color: var(--color-phosphor-muted);">
              <p>No phases yet.</p>
              <p class="text-xs mt-1">Run: <code style="color: var(--color-phosphor);">bd new phase "Phase 1" --in "{detail.node.title}"</code></p>
            </div>
          {:else}
            <div class="flex flex-col gap-3">
              {#each detail.children as phase}
                <PhaseCard {phase} />
              {/each}
            </div>
          {/if}
        {:else}
          <div class="flex flex-col gap-3">
            {#if detail.blocks.length === 0 && detail.blocked_by.length === 0 && detail.related.length === 0}
              <p class="text-sm" style="color: var(--color-phosphor-muted);">No connections yet.</p>
            {:else}
              {#each detail.blocks as edge}
                <ConnectionBadge {edge} currentNodeId={id} direction="blocks" />
              {/each}
              {#each detail.blocked_by as edge}
                <ConnectionBadge {edge} currentNodeId={id} direction="blocked_by" />
              {/each}
              {#each detail.related as edge}
                <ConnectionBadge {edge} currentNodeId={id} direction="related" />
              {/each}
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
