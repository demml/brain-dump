<script lang="ts">
  import type { Tag } from "$lib/types";
  interface Props { tag: Tag; }
  let { tag }: Props = $props();
</script>

<span
  class="inline-flex items-center gap-1 px-2 py-0.5 text-xs border rounded-sm"
  style="
    border-color: var(--color-border);
    background: color-mix(in oklch, {tag.color} 15%, transparent);
    color: var(--color-phosphor-dim);
  "
>
  <span class="w-1.5 h-1.5 rounded-full shrink-0" style="background: {tag.color};"></span>
  {tag.name}
</span>
