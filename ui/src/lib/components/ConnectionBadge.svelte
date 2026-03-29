<script lang="ts">
  import type { Edge } from "$lib/types";

  interface Props {
    edge: Edge;
    currentNodeId: string;
    direction: "blocks" | "blocked_by" | "related";
  }
  let { edge, currentNodeId, direction }: Props = $props();

  let otherId = $derived(edge.source_id === currentNodeId ? edge.target_id : edge.source_id);

  let label = $derived(
    direction === "blocks" ? `→ blocks` :
    direction === "blocked_by" ? `← blocked by` :
    `↔ related`
  );

  let color = $derived(
    direction === "blocks" || direction === "blocked_by"
      ? "var(--color-danger)"
      : "var(--color-phosphor-dim)"
  );
</script>

<span
  class="inline-flex items-center gap-1 px-2 py-0.5 text-xs border"
  style="border-color: {color}; color: {color};"
>
  {label} <code class="font-mono text-xs">{otherId.slice(0, 8)}…</code>
</span>
