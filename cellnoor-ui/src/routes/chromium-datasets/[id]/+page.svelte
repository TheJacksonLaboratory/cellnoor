<script lang="ts">
  import { DATE_FORMATTER } from "$lib/date";
  import { type FileNode } from "$lib/file-tree";
  import type { SpecimenSummary } from "cellnoor-client";
  import * as FileViewer from "../../../components/FileViewer";
  import NiceTable from "../../../components/NiceTable.svelte";
  import LibraryTypeBadges from "../../LibraryTypeBadges.svelte";

  const { data } = $props();

  let activeFile: FileNode | null = $state(null);

  // TBH, we could make this much simpler by just title-casing every property name and just making sure that if it's the ID, name, or date, we do some special processing!
  const specimenTableFieldNames = [
    "Specimen ID",
    "Specimen Name",
    "Date Received",
    "Tissue",
    "Embedded In",
    "Fixative",
    "Thermal Preservation Method",
  ];

  function extractSpecimenDatum(
    fieldName: string,
    {
      readable_id,
      name,
      received_at,
      tissue,
      embedded_in,
      fixative,
      thermal_preservation_method,
    }: SpecimenSummary,
  ) {
    return {
      "Specimen ID": readable_id,
      "Specimen Name": name,
      "Date Received": DATE_FORMATTER.format(new Date(received_at)),
      Tissue: tissue,
      "Embedded In": embedded_in,
      Fixative: fixative,
      "Thermal Preservation Method": thermal_preservation_method,
    }[fieldName];
  }
</script>

{#if data.error}
  <p>Something went wrong</p>
{:else}
  <div class="flex flex-row w-full">
    <div class="border-r overflow-y-auto h-screen sticky top-0 shrink-0">
      <FileViewer.Tree
        fileTree={data.fileTree}
        onselect={(node) => {
          activeFile = node;
        }}
      />
    </div>

    <div class="flex flex-col w-screen p-2 gap-2">
      <h1 class="text-2xl">
        {data.dataset.project.name} / <span class="font-semibold">{data.dataset.name}</span>
      </h1>
      <p>
        {data.dataset.assay.name}
        <span class="font-extralight">({data.dataset.assay.chemistry_version})</span>
      </p>

      <p class="text-sm">
        Delivered on {DATE_FORMATTER.format(new Date(data.dataset.delivered_at))}
      </p>
      <NiceTable
        data={data.specimens!}
        fieldNames={specimenTableFieldNames}
        extractDatum={extractSpecimenDatum}
      />
      <div class="divider m-0"></div>

      <FileViewer.View file={activeFile} />
    </div>
  </div>
{/if}
