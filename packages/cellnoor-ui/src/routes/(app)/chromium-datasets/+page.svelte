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
			<MultiSelect name="specimen.type" fieldLabel="Type" options={specimenTypes} />
		</fieldset>
	{/snippet}

	<table>
		<tbody>
			{#each datasets as ds (ds.id)}
				<tr><td>{ds.id}</td></tr>
				<tr><td>{ds.specimens.map((s) => s.species).join('\n')}</td></tr>
			{/each}
		</tbody>
	</table>
</FilterLayout>
