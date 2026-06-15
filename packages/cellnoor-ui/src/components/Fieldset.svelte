<script lang="ts">
  import type { Snippet } from "svelte";

  const { children, name }: { children: Snippet; name: string } = $props();
</script>

<fieldset class="fieldset bg-base-200 border rounded-box p-2">
  <legend class="fieldset-legend font-bold text-lg">
    {name}
  </legend>
  {@render children()}
</fieldset>
