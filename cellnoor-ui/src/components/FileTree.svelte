<script lang="ts">
  import type { DirectoryNode, FileNode } from "$lib/file-tree";

  const { fileTree }: { fileTree: (DirectoryNode | FileNode)[] } = $props();
</script>

<ul></ul>
