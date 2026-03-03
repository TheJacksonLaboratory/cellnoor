<script lang="ts">
  import { libraryTypeMap } from "$lib/string-maps";
  import type { LibraryType } from "cellnoor-client";

  const {
    libraryTypes,
  }: {
    libraryTypes: (LibraryType | null)[] | undefined;
  } = $props();
</script>

<div class="flex flex-row flex-wrap gap-1">
  {#if libraryTypes !== undefined}
    {#each libraryTypes.filter((libType) => libType !== null) as libraryType}
      <span class="badge badge-secondary font-bold badge-sm h-auto"
        >{libraryTypeMap.get(libraryType)}</span
      >
    {/each}
  {/if}
</div>
