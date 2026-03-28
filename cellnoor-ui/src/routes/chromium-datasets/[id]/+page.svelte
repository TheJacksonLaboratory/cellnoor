<script lang="ts">
  import { type FileNode } from "$lib/file-tree";
  import * as FileViewer from "../../../components/FileViewer";

  const { data } = $props();
  let activeFile: FileNode | null = $state(null);
</script>

<div class="flex flex-row w-full">
  <FileViewer.Tree
    fileTree={data.fileTree}
    onselect={(node) => {
      activeFile = node;
    }}
  />
  <FileViewer.View file={activeFile} />
</div>
