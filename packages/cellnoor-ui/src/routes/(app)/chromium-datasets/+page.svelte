<script lang="ts">
	let { data } = $props();

	const { datasets, datasetIds } = $derived(data);

	let selected: string[] = $state([]);
	let firstSelected = $derived(selected.at(0));
</script>

<table>
	<thead>
		<tr>
			<th>
				<input
					type="checkbox"
					checked={datasetIds.length > 0 && selected.length === datasetIds.length}
					onchange={(e) => (selected = e.currentTarget.checked ? datasetIds : [])}
					aria-label="Select all"
				/>
			</th>
			<th>Name</th>
			<th>Delivered</th>
			<th>Assay</th>
			<th>Specimens</th>
		</tr>
	</thead>
	<tbody>
		{#each datasets as { id, name, delivered_at, assay, specimens } (id)}
			<tr>
				<td>
					<input type="checkbox" bind:group={selected} value={id} aria-label="Select {name}" />
				</td>
				<td>{name}</td>
				<td>{new Date(delivered_at).toLocaleDateString()}</td>
				<td>{assay.name} ({assay.chemistry_version})</td>
				<td>{specimens.map((s) => s.name).join('\n')}</td>
			</tr>
		{/each}
	</tbody>
</table>
