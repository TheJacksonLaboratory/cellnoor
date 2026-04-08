<script lang="ts">
  import { isDirectory, type DirectoryNode, type FileNode } from "$lib/file-tree";
  import { File, Folder } from "@lucide/svelte";
  import Tree from "./Tree.svelte";

  const {
    fileTree,
    onselect,
  }: {
    fileTree: (DirectoryNode | FileNode)[];
    onselect: (node: FileNode) => void;
  } = $props();
</script>

<ul class="menu">
  {#each fileTree as node (node.name)}
    {#if isDirectory(node)}
      <li>
        <details open>
          <summary>
            <Folder />
            {node.name}</summary
          >
          <Tree fileTree={node.children} {onselect} />
        </details>
      </li>
    {:else}
      <li>
        <button
          onblur={({ currentTarget }) => currentTarget.removeAttribute("class")}
          onclick={({ currentTarget }) => {
            currentTarget.setAttribute("class", "border border-success");
            onselect(node);
          }}
        >
          <File />
          {node.name}
        </button>
      </li>
    {/if}
  {/each}
</ul>
