<script lang="ts">
  import { isHtmlFile, type FileNode } from "$lib/file-tree";
  import NiceTable from "../NiceTable.svelte";

  const { file }: { file: FileNode | null } = $props();
</script>

{#if !file}{:else if isHtmlFile(file)}
  <iframe class="grow border rounded min-h-screen" src={file.src} title={file.name}></iframe>
{:else if Array.isArray(file.content)}
  <NiceTable
    fieldNames={Object.keys(file.content[0]!)}
    data={file.content}
    extractDatum={(fieldName, row) => row[fieldName]}
  />
{:else}{/if}
