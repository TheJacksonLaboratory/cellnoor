<script lang="ts">
  import { isHtmlFile, type FileNode } from "$lib/file-tree";

  const { file }: { file: FileNode | null } = $props();
</script>

{#if file && isHtmlFile(file)}
  <iframe class="grow" src={file.src} title={file.name}></iframe>
{/if}
