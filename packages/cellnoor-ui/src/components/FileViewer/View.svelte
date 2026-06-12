<script lang="ts">
  import { isHtmlFile, type FileNode } from "$lib/file-tree";
  import NiceTable from "../NiceTable.svelte";

  const { file }: { file: FileNode | null } = $props();
</script>

{#if Array.isArray(file.content) && file.content.length != 1}
  <NiceTable
    fieldNames={Object.keys(file.content[0]!)}
    data={file.content}
    extractDatum={(fieldName, row) => row[fieldName]}
  />
{:else if Array.isArray(file.content)}
  <NiceTable
    fieldNames={["Metric Name", "Value"]}
    data={Object.entries(file.content[0]!).map(([key, value]) => {
      return { "Metric Name": key, Value: value } as Record<string, unknown>;
    })}
    extractDatum={(fieldName, row) => row[fieldName]}
  />
{:else}
  <NiceTable
    fieldNames={["Metric Name", "Value"]}
    data={Object.entries(file.content).map(([key, value]) => {
      return { "Metric Name": key, Value: value } as Record<string, unknown>;
    })}
    extractDatum={(fieldName, row) => row[fieldName]}
  />
{/if}
