<script lang="ts">
  import type { LibraryType } from "../lib/cellnoor-types";

  const {
    libraryTypes,
  }: {
    libraryTypes: (LibraryType | null)[] | undefined;
  } = $props();
</script>

<div class="flex flex-row flex-wrap gap-1">
  {#if libraryTypes !== undefined}
    {#each libraryTypes.filter((libType) => libType !== null) as libraryType (libraryType)}
      <span class="badge badge-secondary font-bold badge-sm h-auto">{libraryType}</span>
    {/each}
  {/if}
</div>
