<script lang="ts">
	import type { FileLeaf } from './file-tree.ts';

	let { file }: { file: FileLeaf } = $props();

	const data = $derived(file.parsed_contents?.data);

	const columns = $derived(Array.isArray(data) ? [...new Set(data.flatMap(Object.keys))] : []);

	const formatCell = (value: unknown) =>
		typeof value === 'object' ? JSON.stringify(value) : value;
</script>

<section class="stack">
	<header class="cluster">
		<h2 class="subheading">{file.name}</h2>
		{#if data}
			<a class="button" href={file.raw_file_link} download>Download</a>
		{:else}
			<a class="button" href={file.raw_file_link} target="_blank">Open in new tab</a>
		{/if}
	</header>

	{#if !data}
		<iframe src={file.raw_file_link} title={file.name}></iframe>
	{:else if Array.isArray(data)}
		<div class="scroll">
			<table>
				<thead>
					<tr>
						{#each columns as column (column)}
							<th scope="col">{column}</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each data as row}
						<tr>
							{#each columns as column (column)}
								<td>{formatCell(row[column])}</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else}
		<div class="scroll">
			<table>
				<tbody>
					{#each Object.entries(data) as [key, value] (key)}
						<tr>
							<th scope="row">{key}</th>
							<td>{formatCell(value)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</section>

<style>
	.scroll {
		overflow-x: auto;
	}

	table {
		border-collapse: collapse;
	}

	th,
	td {
		padding: var(--space-xs) var(--space-sm);
		border: var(--border);
		text-align: start;
	}

	th {
		background: var(--color-hover);
	}

	iframe {
		inline-size: 100%;
		block-size: 80dvh;
		border: var(--border);
		border-radius: var(--radius);
	}
</style>
