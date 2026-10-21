<script lang="ts">
	import { JsonTreeView } from '@ark-ui/svelte/json-tree-view';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import type { FileLeaf } from './file-tree.ts';

	let { file }: { file: FileLeaf } = $props();

	const format = $derived(file.parsed_contents?.format);
	const data = $derived(file.parsed_contents?.data);

	const rows = $derived(Array.isArray(data) ? data : data ? [data] : []);
	const columns = $derived([...new Set(rows.flatMap(Object.keys))]);

	const formatCell = (value: unknown) =>
		typeof value === 'object' ? JSON.stringify(value) : value;
</script>

<section class="stack card card-body">
	<header class="cluster">
		<h2 class="heading">{file.name}</h2>
		{#if data}
			<a class="button" href={file.raw_file_link} rel="external" download>Download</a>
		{:else}
			<a class="button" href={file.raw_file_link} rel="external" target="_blank">Open in new tab</a>
		{/if}
	</header>

	{#if !data}
		<iframe src={file.raw_file_link} title={file.name}></iframe>
	{:else if format === 'json'}
		<JsonTreeView.Root class="json-tree" {data} defaultExpandedDepth={1}>
			<JsonTreeView.Tree>
				{#snippet arrow()}
					<ChevronRight size={16} />
				{/snippet}
			</JsonTreeView.Tree>
		</JsonTreeView.Root>
	{:else}
		<div class="scroll-x">
			<table>
				<thead>
					<tr>
						{#each columns as column (column)}
							<th scope="col">{column}</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each rows as row (row)}
						<tr>
							{#each columns as column (column)}
								<td>{formatCell(row[column])}</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</section>

<style>
	iframe {
		inline-size: 100%;
		block-size: 80dvh;
		border: none;
	}

	/* Ark renders these parts inside its own components, so scoped selectors can't reach them */
	:global {
		.json-tree {
			font-family: ui-monospace, monospace;

			[data-part='branch-control'],
			[data-part='item'] {
				display: flex;
				align-items: center;
				padding-inline-start: calc((var(--depth) - 1) * var(--space-md));
				cursor: pointer;

				&:hover {
					background: var(--color-hover);
				}
			}

			/* Lines leaf items up with branch text, past the 16px arrow */
			[data-part='item'] {
				padding-inline-start: calc((var(--depth) - 1) * var(--space-md) + 16px);
			}

			[data-part='branch-indicator'] {
				display: flex;
			}

			[data-kind='colon'] {
				margin-inline-end: var(--space-xs);
			}
		}
	}
</style>
