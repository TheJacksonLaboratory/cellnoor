<script lang="ts">
	import { TreeView, createTreeCollection } from '@ark-ui/svelte/tree-view';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import FileIcon from '@lucide/svelte/icons/file';
	import Folder from '@lucide/svelte/icons/folder';
	import { type FileLeaf, type FileNode } from './file-tree.ts';

	let { files, selected = $bindable() }: { files: FileNode[]; selected?: FileLeaf } = $props();

	const iconSize = '1em';

	const collection = $derived(
		createTreeCollection<FileNode>({
			nodeToValue: (node) => node.path,
			nodeToString: (node) => node.name,
			rootNode: { type: 'directory', path: '', name: '', children: files }
		})
	);
</script>

<TreeView.Root
	class="file-explorer card"
	{collection}
	selectedValue={selected ? [selected.path] : []}
	onSelectionChange={({ selectedNodes: [node] }) => {
		if (node?.type === 'file') {
			selected = node;
		}
	}}
>
	<TreeView.Label class="subheading card-header">Files</TreeView.Label>
	<TreeView.Tree class="card-body">
		{#each files as node, index (node.path)}
			{@render renderNode(node, [index])}
		{/each}
	</TreeView.Tree>
</TreeView.Root>

{#snippet renderNode(node: FileNode, indexPath: number[])}
	<TreeView.NodeProvider {node} {indexPath}>
		{#if node.type === 'directory'}
			<TreeView.Branch>
				<TreeView.BranchControl>
					<TreeView.BranchIndicator>
						<ChevronRight size={iconSize} />
					</TreeView.BranchIndicator>
					<Folder size={iconSize} />
					<TreeView.BranchText>{node.name}</TreeView.BranchText>
				</TreeView.BranchControl>
				<TreeView.BranchContent>
					{#each node.children as child, index (child.path)}
						{@render renderNode(child, [...indexPath, index])}
					{/each}
				</TreeView.BranchContent>
			</TreeView.Branch>
		{:else}
			<TreeView.Item>
				<FileIcon size={iconSize} />
				<TreeView.ItemText>{node.name}</TreeView.ItemText>
			</TreeView.Item>
		{/if}
	</TreeView.NodeProvider>
{/snippet}

<style>
	/* Ark renders these parts inside its own components, so scoped selectors can't reach them */
	:global {
		.file-explorer {
			[data-scope='tree-view'][data-part='item'],
			[data-scope='tree-view'][data-part='branch-control'] {
				display: flex;
				align-items: center;
				gap: var(--space-xs);
				padding-block: var(--space-xs);
				padding-inline-start: calc(var(--space-sm) + (var(--depth) - 1) * var(--space-md));
				padding-inline-end: var(--space-sm);
				border-radius: var(--radius);
				cursor: pointer;
				overflow-wrap: anywhere;

				&:hover {
					background: var(--color-hover);
				}

				&[data-selected] {
					background: var(--color-hover);
					color: var(--color-primary);
				}
			}

			/* Add the width of the chevron */
			[data-scope='tree-view'][data-part='item'] {
				padding-inline-start: calc(1em + var(--space-sm) + (var(--depth) - 1) * var(--space-md));
			}

			[data-scope='tree-view'][data-part='branch-indicator'] {
				display: flex;
			}
		}
	}
</style>
