<script lang="ts">
  import type { Snippet } from "svelte";

  const { badgeText, children }: { badgeText: string; children: Snippet } = $props();
</script>

<span class="badge badge-secondary font-bold badge-sm h-auto"
  >{badgeText}
  {@render children()}</span
>
