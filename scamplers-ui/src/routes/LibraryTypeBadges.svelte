<script lang="ts">
  import { libraryTypeMap } from "$lib/string-maps";
  import type { LibraryType } from "scamplers-types/LibraryType";

  const { libraryTypes }: {
    libraryTypes: (LibraryType | null)[] | undefined;
  } = $props();
</script>

<div class="flex flex-row flex-wrap gap-1">
  {#if libraryTypes !== undefined}
    {#each     libraryTypes.filter((libType) => {
      return libType !== null;
    }) as
      libraryType
    }
      <span class="badge badge-neutral badge-sm h-auto">{
        libraryTypeMap.get(libraryType)
      }</span>
    {/each}
  {/if}
</div>
