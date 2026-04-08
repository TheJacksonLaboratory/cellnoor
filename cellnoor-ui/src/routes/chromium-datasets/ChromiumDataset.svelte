<!-- I cannot believe how shitty this is -->
<script lang="ts">
  import { DATE_FORMATTER } from "$lib/date";
  import type { ChromiumDataset } from "cellnoor-client";
  import LibraryTypeBadges from "../LibraryTypeBadges.svelte";

  const {
    chromiumDataset,
  }: {
    chromiumDataset: ChromiumDataset & {
      files: Map<string, string[]>;
    };
  } = $props();
  const { links, name, assay, delivered_at, files } = $derived(chromiumDataset);
</script>

<div class="flex flex-row p-4 mb-4 border border-neutral rounded-box place-content-between">
  <div class="flex flex-col gap-1">
    <!-- prettier-ignore -->
    <a class="link link-primary link-hover text-xl font-semibold max-w-fit" href={links.self}>{name}</a><!-- eslint-disable-line svelte/no-navigation-without-resolve -->
    <p>
      {assay.name}
      <span class="font-extralight">({assay.chemistry_version})</span>
    </p>
    {#if assay.library_types}
      <LibraryTypeBadges libraryTypes={assay.library_types} />
    {/if}
    <span class="text-sm mt-2">Delivered on {DATE_FORMATTER.format(new Date(delivered_at))}</span>
  </div>
</div>
