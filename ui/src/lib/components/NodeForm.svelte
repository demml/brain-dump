<script lang="ts">
  import type { Node } from "$lib/types";
  import { createNode, updateNode } from "$lib/tauri";
  import { marked } from "marked";

  interface Props {
    /** Node type for new nodes */
    nodeType: "project" | "phase" | "task";
    /** Parent node ID (for phase/task creation) */
    parentId?: string;
    /** Existing node — if provided, this is an edit form */
    node?: Node;
    /** Called after successful save with the created/updated node */
    onSave: (node: Node) => void;
    /** Called when user cancels */
    onCancel: () => void;
  }

  let { nodeType, parentId, node, onSave, onCancel }: Props = $props();

  // Initialize from existing node or empty
  let title = $state(node?.title ?? "");
  let description = $state(node?.description ?? "");
  let showPreview = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);

  let previewHtml = $derived(showPreview ? (marked.parse(description, { async: false }) as string) : "");

  let isValid = $derived(title.trim().length > 0);

  async function save() {
    if (!isValid || saving) return;
    saving = true;
    error = null;
    try {
      let saved: Node;
      if (node) {
        // Edit existing
        saved = await updateNode(node.id, title.trim(), description);
      } else {
        // Create new
        saved = await createNode(nodeType, title.trim(), parentId);
        // Update description if user changed it from template
        if (description !== saved.description) {
          saved = await updateNode(saved.id, undefined, description);
        }
      }
      onSave(saved);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onCancel();
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") save();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<!-- Backdrop -->
<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center p-4"
  style="background: oklch(0% 0 0 / 0.6);"
  onclick={(e) => { if (e.target === e.currentTarget) onCancel(); }}
>
  <!-- Modal -->
  <div
    class="w-full max-w-2xl border-2 flex flex-col"
    style="
      background: var(--color-bg-card);
      border-color: var(--color-border-bright);
      box-shadow: 6px 6px 0 0 oklch(40% 0.08 150 / 0.3);
      max-height: 90vh;
    "
  >
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b-2" style="border-color: var(--color-border);">
      <h2 class="font-bold" style="color: var(--color-phosphor-bright);">
        {node ? `Edit ${nodeType}` : `New ${nodeType}`}
      </h2>
      <button
        class="text-lg px-2"
        style="color: var(--color-phosphor-muted);"
        onclick={onCancel}
      >×</button>
    </div>

    <!-- Form body -->
    <div class="flex flex-col gap-4 p-4 overflow-auto flex-1">
      <!-- Title field -->
      <div>
        <label class="block text-xs mb-1" style="color: var(--color-phosphor-dim);">Title</label>
        <input
          type="text"
          bind:value={title}
          class="w-full px-3 py-2 border-2 outline-none text-sm"
          style="
            background: var(--color-bg-input);
            border-color: var(--color-border);
            color: var(--color-phosphor);
          "
          placeholder="Enter title..."
          autofocus
        />
      </div>

      <!-- Description field with preview toggle -->
      <div class="flex-1">
        <div class="flex items-center justify-between mb-1">
          <label class="text-xs" style="color: var(--color-phosphor-dim);">Description (markdown)</label>
          <button
            class="text-xs"
            style="color: {showPreview ? 'var(--color-phosphor)' : 'var(--color-phosphor-muted)'};"
            onclick={() => showPreview = !showPreview}
          >
            {showPreview ? "← Edit" : "Preview →"}
          </button>
        </div>

        {#if showPreview}
          <div
            class="border-2 p-3 min-h-40 text-sm overflow-auto"
            style="
              border-color: var(--color-border);
              background: var(--color-bg);
              color: var(--color-phosphor-dim);
            "
          >
            {@html previewHtml}
          </div>
        {:else}
          <textarea
            bind:value={description}
            class="w-full px-3 py-2 border-2 outline-none text-sm font-mono resize-none"
            style="
              background: var(--color-bg-input);
              border-color: var(--color-border);
              color: var(--color-phosphor);
              min-height: 10rem;
            "
            placeholder="Describe this {nodeType}..."
          ></textarea>
        {/if}
      </div>

      <!-- Error message -->
      {#if error}
        <p class="text-sm" style="color: var(--color-danger);">{error}</p>
      {/if}
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-between p-4 border-t-2" style="border-color: var(--color-border);">
      <span class="text-xs" style="color: var(--color-phosphor-muted);">⌘↵ to save · Esc to cancel</span>
      <div class="flex gap-3">
        <button
          class="px-4 py-1.5 text-sm border-2 transition-colors"
          style="border-color: var(--color-border); color: var(--color-phosphor-dim);"
          onclick={onCancel}
        >
          Cancel
        </button>
        <button
          class="px-4 py-1.5 text-sm border-2 transition-colors"
          style="
            border-color: {isValid ? 'var(--color-phosphor)' : 'var(--color-border)'};
            color: {isValid ? 'var(--color-bg)' : 'var(--color-phosphor-muted)'};
            background: {isValid ? 'var(--color-phosphor)' : 'transparent'};
          "
          onclick={save}
          disabled={!isValid || saving}
        >
          {saving ? "Saving…" : "Save"}
        </button>
      </div>
    </div>
  </div>
</div>
