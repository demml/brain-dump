<script lang="ts">
  import type { Node } from "$lib/types";
  import { listNodes } from "$lib/tauri";
  import ProgressBar from "./ProgressBar.svelte";
  import TaskItem from "./TaskItem.svelte";
  import { onMount } from "svelte";

  interface Props { phase: Node; }
  let { phase }: Props = $props();

  let expanded = $state(false);
  let tasks = $state<Node[]>([]);
  let tasksLoaded = $state(false);

  async function loadTasks() {
    if (!tasksLoaded) {
      tasks = await listNodes("task", phase.id);
      tasksLoaded = true;
    }
  }

  async function toggle() {
    expanded = !expanded;
    if (expanded) await loadTasks();
  }

  async function refreshTasks() {
    tasks = await listNodes("task", phase.id);
  }

  let completedCount = $derived(tasks.filter(t => t.status === "completed").length);
</script>

<div
  class="border-2 transition-all"
  style="border-color: var(--color-border); background: var(--color-bg-card); box-shadow: var(--shadow-hard);"
>
  <!-- Phase header (clickable) -->
  <button
    class="w-full flex items-center justify-between p-3 text-left"
    onclick={toggle}
  >
    <div class="flex items-center gap-2">
      <span class="text-xs" style="color: var(--color-phosphor-dim);">{expanded ? "▼" : "▶"}</span>
      <span class="font-bold text-sm" style="color: var(--color-phosphor-bright);">{phase.title}</span>
    </div>
    <div class="flex items-center gap-3 shrink-0">
      {#if tasksLoaded}
        <span class="text-xs" style="color: var(--color-phosphor-muted);">{completedCount}/{tasks.length}</span>
      {/if}
      <span
        class="text-xs px-1.5 py-0.5 border"
        style="border-color: var(--color-border); color: var(--color-phosphor-muted);"
      >
        {phase.status}
      </span>
    </div>
  </button>

  <!-- Progress bar (always visible if loaded) -->
  {#if tasksLoaded && tasks.length > 0}
    <div class="px-3 pb-2">
      <ProgressBar completed={completedCount} total={tasks.length} />
    </div>
  {/if}

  <!-- Task list (expanded) -->
  {#if expanded}
    <div class="border-t" style="border-color: var(--color-border);">
      {#if tasks.length === 0}
        <p class="text-xs p-3" style="color: var(--color-phosphor-muted);">No tasks yet.</p>
      {:else}
        {#each tasks as task}
          <TaskItem {task} onStatusChange={refreshTasks} />
        {/each}
      {/if}
    </div>
  {/if}
</div>
