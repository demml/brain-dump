<script lang="ts">
  import type { Node } from "$lib/types";
  import { updateNode } from "$lib/tauri";

  interface Props {
    task: Node;
    onStatusChange?: () => void;
  }
  let { task, onStatusChange }: Props = $props();

  let loading = $state(false);

  async function toggleComplete() {
    if (loading) return;
    loading = true;
    try {
      const newStatus = task.status === "completed" ? "active" : "completed";
      await updateNode(task.id, undefined, undefined, newStatus);
      onStatusChange?.();
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex items-center gap-3 py-1.5 px-2 group">
  <button
    class="w-4 h-4 border flex items-center justify-center shrink-0 transition-colors"
    style="
      border-color: {task.status === 'completed' ? 'var(--color-phosphor)' : 'var(--color-border)'};
      background: {task.status === 'completed' ? 'var(--color-phosphor)' : 'transparent'};
      color: var(--color-bg);
    "
    onclick={toggleComplete}
    disabled={loading}
    title={task.status === "completed" ? "Mark incomplete" : "Mark complete"}
  >
    {#if task.status === "completed"}✓{/if}
  </button>
  <span
    class="text-sm flex-1"
    style="
      color: {task.status === 'completed' ? 'var(--color-phosphor-muted)' : 'var(--color-phosphor)'};
      text-decoration: {task.status === 'completed' ? 'line-through' : 'none'};
    "
  >
    {task.title}
  </span>
  {#if task.status === "archived"}
    <span class="text-xs" style="color: var(--color-phosphor-muted);">archived</span>
  {/if}
</div>
