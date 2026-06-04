<script lang="ts">
  import { isDirectory, type DirectoryNode, type FileNode } from "$lib/file-tree";
  import { File, Folder } from "@lucide/svelte";
  import Tree from "./Tree.svelte";

  let {
    fileTree,
    onselect,
    selected = $bindable<FileNode | null>(null),
  }: {
    fileTree: (DirectoryNode | FileNode)[];
    onselect: (node: FileNode) => void;
    selected?: FileNode | null;
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
          <Tree fileTree={node.children} {onselect} bind:selected />
        </details>
      </li>
    {:else}
      <li>
        <button
          class={selected?.src === node.src ? "menu-active" : ""}
          onclick={() => {
            selected = node;
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
