<script lang="ts">
  import { marked } from "marked";

  interface Props { content: string; }
  let { content }: Props = $props();

  let html = $derived(marked.parse(content, { async: false }));
</script>

<div class="markdown-content">{@html html}</div>

<style>
  .markdown-content :global(h1),
  .markdown-content :global(h2),
  .markdown-content :global(h3) {
    color: var(--color-phosphor-bright);
    margin-top: 1rem;
    margin-bottom: 0.5rem;
    font-weight: 700;
  }
  .markdown-content :global(p) {
    color: var(--color-phosphor-dim);
    line-height: 1.6;
    margin-bottom: 0.75rem;
  }
  .markdown-content :global(code) {
    color: var(--color-phosphor);
    background: var(--color-bg-input);
    padding: 0.1rem 0.3rem;
    border-radius: 2px;
    font-size: 0.85em;
  }
  .markdown-content :global(a) {
    color: var(--color-phosphor);
    text-decoration: underline;
  }
  .markdown-content :global(ul),
  .markdown-content :global(ol) {
    color: var(--color-phosphor-dim);
    padding-left: 1.5rem;
    margin-bottom: 0.75rem;
  }
  .markdown-content :global(li) {
    margin-bottom: 0.25rem;
  }
  /* HTML comments as placeholder text */
  .markdown-content :global(p:has(> *:only-child:empty)) {
    display: none;
  }
</style>
