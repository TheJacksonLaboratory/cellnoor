<script lang="ts">
	import { TreeView, createTreeCollection } from '@ark-ui/svelte/tree-view';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import FileIcon from '@lucide/svelte/icons/file';
	import Folder from '@lucide/svelte/icons/folder';
	import { withValues, type FileLeaf, type FileNode, type ValuedFileNode } from './file-tree.ts';

	let { files, selected = $bindable() }: { files: FileNode[]; selected?: FileLeaf } = $props();

	const nodes = $derived(withValues(files));

	const collection = () =>
		createTreeCollection<ValuedFileNode>({
			nodeToValue: (node) => node.value,
			nodeToString: (node) => node.name,
			rootNode: { type: 'directory', name: '', value: '/', children: nodes }
		});
</script>

<TreeView.Root
	lazyMount
	class="file-explorer"
	collection={collection()}
	onSelectionChange={({ selectedNodes: [node] }) => {
		if (node?.type === 'file') {
			selected = node;
		}
	}}
>
	<TreeView.Label class="subheading">Files</TreeView.Label>
	<TreeView.Tree>
		{#each nodes as node, index (node.value)}
			{@render renderNode(node, [index])}
		{/each}
	</TreeView.Tree>
</TreeView.Root>

{#snippet renderNode(node: ValuedFileNode, indexPath: number[])}
	<TreeView.NodeProvider {node} {indexPath}>
		{#if node.type === 'directory'}
			<TreeView.Branch>
				<TreeView.BranchControl>
					<TreeView.BranchIndicator>
						<ChevronRight size={16} />
					</TreeView.BranchIndicator>
					<Folder size={16} />
					<TreeView.BranchText>{node.name}</TreeView.BranchText>
				</TreeView.BranchControl>
				<TreeView.BranchContent>
					{#each node.children as child, index (child.value)}
						{@render renderNode(child, [...indexPath, index])}
					{/each}
				</TreeView.BranchContent>
			</TreeView.Branch>
		{:else}
			<TreeView.Item>
				<FileIcon size={16} />
				<TreeView.ItemText>{node.name}</TreeView.ItemText>
			</TreeView.Item>
		{/if}
	</TreeView.NodeProvider>
{/snippet}

<style>
	/* Ark renders these parts inside its own components, so scoped selectors can't reach them */
	:global {
		.file-explorer {
			display: flex;
			flex-direction: column;
			gap: var(--space-sm);

			[data-part='item'],
			[data-part='branch-control'] {
				display: flex;
				align-items: center;
				gap: var(--space-xs);
				padding-block: var(--space-xs);
				padding-inline: calc(var(--space-sm) + (var(--depth) - 1) * var(--space-md)) var(--space-sm);
				border-radius: var(--radius);
				cursor: pointer;
				overflow-wrap: anywhere;

				&:hover {
					background: var(--color-hover);
				}

				&[data-selected] {
					background: var(--color-hover);
					color: var(--color-primary);
					font-weight: bold;
				}
			}

			[data-part='branch-indicator'] {
				display: flex;
				transition: rotate 150ms ease;

				&[data-state='open'] {
					rotate: 90deg;
				}
			}
		}
	}
</style>
