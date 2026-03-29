<script lang="ts">
  interface Props {
    completed: number;
    total: number;
    maxSegments?: number;
  }
  let { completed, total, maxSegments = 8 }: Props = $props();

  let segments = $derived(Math.min(total, maxSegments));
  let completedSegments = $derived(total > 0 ? Math.round((completed / total) * segments) : 0);
</script>

<div class="flex gap-1 items-center">
  {#each { length: segments } as _, i}
    <div
      class="h-1.5 flex-1 rounded-sm"
      style="background: {i < completedSegments ? 'var(--color-phosphor)' : 'var(--color-border)'};"
    ></div>
  {/each}
  <span class="text-xs ml-1 shrink-0" style="color: var(--color-phosphor-muted);">{completed}/{total}</span>
</div>
