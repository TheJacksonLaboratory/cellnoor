<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import DateRange from '#lib/filters/DateRange.svelte';
	import FilterLayout from '#lib/filters/FilterLayout.svelte';
	import MultiSelect from '#lib/filters/MultiSelect.svelte';
	import MultiText from '#lib/filters/MultiText.svelte';
	import {
		blockEmbeddingMatrixValues,
		controlledRateFreezingValues,
		flashFreezingValues,
		speciesValues,
		specimenTypeValues
	} from 'cellnoor-client/cellnoor-types.ts';

	const toItem = (value: string) => [value, value] as [string, string];

	let { data } = $props();

	const { datasets, datasetIds, projects, assays } = $derived(data);

	const projectIdNamePairs = $derived(projects.map((p) => [p.id, p.name] as [string, string]));
	const deduplicatedAssayNames = $derived([...new Set(assays.map((a) => a.name))]);

	let selected: string[] = $state([]);

	afterNavigate(() => (selected = []));
</script>

<FilterLayout>
	{#snippet filters()}
		<fieldset>
			<legend>Specimen Information</legend>
			<MultiText label="Name" name="specimen_name" />
			<MultiSelect label="Species" name="species" items={speciesValues.map(toItem)} />
			<DateRange label="Received" name="received" />
			<MultiSelect label="Type" name="specimen_type" items={specimenTypeValues.map(toItem)} />
			<MultiSelect
				label="Embedding matrix"
				name="embedded_in"
				items={blockEmbeddingMatrixValues.map(toItem)}
			/>
			<MultiSelect
				label="Thermal preservation method"
				name="thermal_preservation"
				items={[...controlledRateFreezingValues, ...flashFreezingValues].map(toItem)}
			/>
		</fieldset>
		<fieldset>
			<legend>Assay</legend>
			<MultiSelect label="Name" name="assay" items={deduplicatedAssayNames.map(toItem)} />
		</fieldset>
		<fieldset>
			<legend>Dataset Information</legend>
			<MultiText label="Name" name="name" />
			<DateRange label="Delivered" name="delivered" />
			<MultiSelect label="Project" name="project" items={projectIdNamePairs} />
		</fieldset>
	{/snippet}

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
</FilterLayout>
