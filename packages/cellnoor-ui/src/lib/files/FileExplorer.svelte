<script lang="ts">
	import { TreeView, createTreeCollection } from '@ark-ui/svelte/tree-view';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import FileIcon from '@lucide/svelte/icons/file';
	import Folder from '@lucide/svelte/icons/folder';
	import X from '@lucide/svelte/icons/x';
	import { withValues, type FileLeaf, type FileNode, type ValuedFileNode } from './file-tree.ts';

	let { files, selected = $bindable() }: { files: FileNode[]; selected?: FileLeaf } = $props();

	const nodes = $derived(withValues(files));

	const collection = $derived(
		createTreeCollection<ValuedFileNode>({
			nodeToValue: (node) => node.value,
			nodeToString: (node) => node.name,
			rootNode: { type: 'directory', name: '', value: '/', children: nodes }
		})
	);
</script>

<TreeView.Root
	class="file-explorer card"
	{collection}
	selectedValue={selected ? [selected.raw_file_link] : []}
	onSelectionChange={({ selectedNodes: [node] }) => {
		if (node?.type === 'file') {
			selected = node;
		}
	}}
>
	<TreeView.Label class="subheading card-header">Files</TreeView.Label>
	<TreeView.Tree class="card-body">
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
				{#if selected?.raw_file_link === node.raw_file_link}
					<!-- Stop propagation so the tree doesn't treat these events as selecting the item -->
					<button
						class="plain-button close"
						aria-label="Close {node.name}"
						onclick={(e) => {
							e.stopPropagation();
							selected = undefined;
						}}
						onkeydown={(e) => e.stopPropagation()}
					>
						<X size={16} />
					</button>
				{/if}
			</TreeView.Item>
		{/if}
	</TreeView.NodeProvider>
{/snippet}

<style>
	.close {
		display: flex;
		margin-inline-start: auto;
	}

	/* Ark renders these parts inside its own components, so scoped selectors can't reach them */
	:global {
		.file-explorer {
			[data-part='item'],
			[data-part='branch-control'] {
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

			[data-part='branch-indicator'] {
				display: flex;
			}
		}
	}
</style>
