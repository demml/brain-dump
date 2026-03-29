<script lang="ts">
  import type { Node, Tag, Progress } from "$lib/types";
  import ProgressBar from "./ProgressBar.svelte";
  import TagBadge from "./TagBadge.svelte";

  interface Props {
    project: Node;
    tags?: Tag[];
    progress?: Progress | null;
  }
  let { project, tags = [], progress = null }: Props = $props();

  let descriptionSnippet = $derived((project.description ?? "").slice(0, 120).trimEnd());
</script>

<a
  href="/project/{project.id}"
  class="block p-4 border-2 transition-all duration-100 no-underline"
  style="
    border-color: var(--color-border);
    background: var(--color-bg-card);
    box-shadow: var(--shadow-hard);
    color: var(--color-phosphor);
  "
  onmouseenter={(e) => {
    const el = e.currentTarget as HTMLElement;
    el.style.transform = 'translate(2px,2px)';
    el.style.boxShadow = 'none';
    el.style.background = 'var(--color-bg-card-hover)';
  }}
  onmouseleave={(e) => {
    const el = e.currentTarget as HTMLElement;
    el.style.transform = '';
    el.style.boxShadow = 'var(--shadow-hard)';
    el.style.background = 'var(--color-bg-card)';
  }}
>
  <!-- Header row -->
  <div class="flex items-start justify-between gap-2 mb-2">
    <h3 class="font-bold text-sm leading-tight" style="color: var(--color-phosphor-bright);">
      {project.title}
    </h3>
    <span
      class="text-xs px-1.5 py-0.5 border shrink-0"
      style="
        border-color: var(--color-border);
        color: {project.status === 'completed' ? 'var(--color-success)' : project.status === 'archived' ? 'var(--color-phosphor-muted)' : 'var(--color-phosphor-dim)'};
      "
    >
      {project.status}
    </span>
  </div>

  <!-- Description -->
  {#if descriptionSnippet}
    <p class="text-xs mb-3 line-clamp-2" style="color: var(--color-phosphor-dim);">
      {descriptionSnippet}
    </p>
  {/if}

  <!-- Progress bar -->
  {#if progress && progress.total > 0}
    <div class="mb-3">
      <ProgressBar completed={progress.completed} total={progress.total} />
    </div>
  {/if}

  <!-- Tags -->
  {#if tags.length > 0}
    <div class="flex flex-wrap gap-1">
      {#each tags as tag}
        <TagBadge {tag} />
      {/each}
    </div>
  {/if}
</a>
