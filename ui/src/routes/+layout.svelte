<script lang="ts">
  import "../app.css";
  let { children } = $props();
</script>

<div class="flex min-h-screen">
  <!-- Sidebar placeholder — will be replaced in Task 10 -->
  <aside class="w-16 border-r-2 border-[var(--color-border)] bg-[var(--color-bg-card)] flex flex-col items-center py-4 gap-4">
    <div class="text-[var(--color-phosphor)] font-bold text-lg">bd</div>
  </aside>

  <!-- Main content -->
  <main class="flex-1 p-6 overflow-auto">
    {@render children()}
  </main>
</div>
