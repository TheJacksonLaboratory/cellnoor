<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import FilterLayout from '#lib/filters/FilterLayout.svelte';
	import MultiSelect from '#lib/filters/MultiSelect.svelte';

	let { data } = $props();

	const {
		datasets,
		projects,
		assays,
		specimenTypes,
		species,
		embeddingMatrices,
		thermalPreservationMethods
	} = $derived(data);

	let selectedDatasets: string[] = $state([]);
	afterNavigate(() => (selectedDatasets = []));
</script>

<FilterLayout>
	{#snippet filters()}
		<fieldset>
			<legend>Specimen Information</legend>
			<MultiSelect name="specimen.species" fieldLabel="Species" options={species} />
			<MultiSelect fieldLabel="Type" name="specimen.type" options={specimenTypes} />
		</fieldset>
	{/snippet}

	<table>
		<thead>
			<tr>
				<th>
					<input
						type="checkbox"
						checked={datasetIds.length > 0 && selectedDatasets.length === datasetIds.length}
						onchange={(e) => (selectedDatasets = e.currentTarget.checked ? datasetIds : [])}
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
						<input
							type="checkbox"
							bind:group={selectedDatasets}
							value={id}
							aria-label="Select {name}"
						/>
					</td>
					<td>{name}</td>
					<td>{new Date(delivered_at).toLocaleDateString()}</td>
					<td>{assay.name} ({assay.chemistry_version})</td>
					<td>{specimens.map((s) => s.name).join('\n')}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</FilterLayout>
