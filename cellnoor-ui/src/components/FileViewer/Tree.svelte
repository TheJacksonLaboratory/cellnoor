<script lang="ts">
  import { isDirectory, type DirectoryNode, type FileNode } from "$lib/file-tree";
  import { File, Folder } from "@lucide/svelte";
  import Tree from "./Tree.svelte";

  const {
    fileTree,
    onselect,
  }: { fileTree: (DirectoryNode | FileNode)[]; onselect: (node: FileNode) => void } = $props();
</script>

<ul class="menu">
  {#each fileTree as node (node.name)}
    {#if isDirectory(node)}
      <li>
        <div>
          <Folder />
          {node.name}
        </div>
        <Tree fileTree={node.children} {onselect} />
      </li>
    {:else}
      <li>
        <button onclick={() => onselect(node)}>
          <File />
          {node.name}
        </button>
      </li>
    {/if}
  {/each}
</ul>
