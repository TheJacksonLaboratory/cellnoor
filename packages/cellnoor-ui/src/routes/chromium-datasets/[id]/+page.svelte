<script lang="ts">
  import type { SpecimenCompact, TaggedSpecimen } from "$lib/cellnoor-types";
  import { DATE_FORMATTER } from "$lib/date";
  import { type FileNode } from "$lib/file-tree";
  import * as FileViewer from "../../../components/FileViewer";
  import NiceTable from "../../../components/NiceTable.svelte";

  const { data } = $props();
  const { dataset, project, error } = data;

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
      multiplexing_tag,
    }: TaggedSpecimen,
  ) {
    return {
      "Specimen ID": readable_id,
      "Specimen Name": name,
      "Date Received": DATE_FORMATTER.format(new Date(received_at)),
      Tissue: tissue,
      "Embedded In": embedded_in,
      Fixative: fixative,
      "Thermal Preservation Method": thermal_preservation_method,
      "Multiplexing Tag": multiplexing_tag ? multiplexing_tag.tag_id : null,
    }[fieldName];
  }
</script>

{#if error}
  {error}
{:else}
  <div class="flex flex-row w-full">
    <div class="flex flex-col w-screen p-2 gap-2">
      <h1 class="text-2xl">
        <!-- TypeScript is so stupid -->
        {project!.name} / <span class="font-semibold">{dataset!.name}</span>
      </h1>
      <p>
        {dataset}
        <span class="font-extralight">({dataset!.assay.chemistry_version})</span>
      </p>
      <p class="text-sm">
        Delivered on {DATE_FORMATTER.format(new Date(dataset!.delivered_at))}
      </p>
      <div class="divider m-0"></div>
      <h1 class="text-lg font-bold">Specimens in this dataset</h1>
      <NiceTable
        data={dataset!.specimens}
        fieldNames={specimenTableFieldNames}
        extractDatum={extractSpecimenDatum}
      />
      <div class="divider m-0"></div>
    </div>
  </div>
{/if}
