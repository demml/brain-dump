<script lang="ts">
  let expanded = $state(false);
  let pinned = $state(false);

  function onMouseEnter() { if (!pinned) expanded = true; }
  function onMouseLeave() { if (!pinned) expanded = false; }
  function togglePin() { pinned = !pinned; expanded = pinned; }
</script>

<aside
  class="flex flex-col h-screen border-r-2 transition-all duration-200 overflow-hidden shrink-0"
  style="
    width: {expanded ? '14rem' : '4rem'};
    border-color: var(--color-border);
    background: var(--color-bg-card);
  "
  onmouseenter={onMouseEnter}
  onmouseleave={onMouseLeave}
  role="navigation"
>
  <!-- Logo/title row -->
  <div class="flex items-center gap-3 px-3 py-4 border-b-2" style="border-color: var(--color-border); min-height: 3.5rem;">
    <span class="text-xl font-bold shrink-0" style="color: var(--color-phosphor-bright);">bd</span>
    {#if expanded}
      <span class="text-sm font-bold whitespace-nowrap" style="color: var(--color-phosphor-dim);">brain-dump</span>
      <button
        class="ml-auto text-xs shrink-0 px-1"
        style="color: {pinned ? 'var(--color-phosphor)' : 'var(--color-phosphor-muted)'};"
        onclick={togglePin}
        title={pinned ? "Unpin sidebar" : "Pin sidebar"}
      >
        {pinned ? "●" : "○"}
      </button>
    {/if}
  </div>

  <!-- Nav items -->
  <nav class="flex flex-col gap-1 p-2 flex-1">
    {#each [
      { icon: "⌂", label: "Home", href: "/" },
      { icon: "◈", label: "Projects", href: "/" },
      { icon: "◇", label: "Tags", href: "/" },
      { icon: "⚙", label: "Settings", href: "/" },
    ] as item}
      <a
        href={item.href}
        class="flex items-center gap-3 px-2 py-2 rounded transition-colors"
        style="color: var(--color-phosphor-dim);"
        onmouseenter={(e) => (e.currentTarget as HTMLElement).style.color = 'var(--color-phosphor)'}
        onmouseleave={(e) => (e.currentTarget as HTMLElement).style.color = 'var(--color-phosphor-dim)'}
      >
        <span class="text-lg shrink-0 w-6 text-center">{item.icon}</span>
        {#if expanded}
          <span class="text-sm whitespace-nowrap">{item.label}</span>
        {/if}
      </a>
    {/each}
  </nav>
</aside>
