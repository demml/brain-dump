<script lang="ts">
  import "../app.css";
  import Sidebar from "$lib/components/Sidebar.svelte";
  let { children } = $props();
</script>

<div class="flex min-h-screen">
  <Sidebar />
  <main class="flex-1 p-6 overflow-auto">
    {@render children()}
  </main>
</div>
