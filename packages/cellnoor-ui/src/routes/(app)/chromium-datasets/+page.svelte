<script lang="ts">
	import { resolve } from '$app/paths';
	import FilteredList from '#lib/FilteredList.svelte';
	import FilterGroup from '#lib/filters/FilterGroup.svelte';
	import MultiSelectFilter from '#lib/filters/MultiSelectFilter.svelte';
	import TagInput from '#lib/filters/TagInput.svelte';
	import DateRange from '#lib/filters/DateRange.svelte';
	import * as options from '#lib/filters/options.ts';
	import ListRow from '#lib/ListRow.svelte';
	import Time from '#lib/Time.svelte';
	import { formatAssayWithMultiplexing, formatSpecies } from '#lib/format.ts';
	import SelectionToolbar from '#lib/SelectionToolbar.svelte';
	import Sort from '#lib/filters/Sort.svelte';
	import PageSize from '#lib/filters/PageSize.svelte';
	import type { ChromiumDatasetField } from 'cellnoor-client/cellnoor-types.ts';
	import { FormFieldName } from './field-parsers.ts';
	import { convertMetricsToFiles } from './convert-metrics.ts';

	let { data } = $props();

	const { datasets, selected, projects, projectNames, tenxAssays, orderBy, limit } = $derived(data);

	const selectedDatasets = $derived(datasets.filter((ds) => selected.has(ds.id)));

	const sortFields = new Map<ChromiumDatasetField, string>([
		['delivered_at', 'Date delivered'],
		['name', 'Name']
	]);

	const SPECIMEN_LIMIT = 8;

	const datasetCount = (n: number) => `${n} ${n === 1 ? 'dataset' : 'datasets'}`;

	function download(blob: Blob, filename: string) {
		const a = document.createElement('a');
		a.href = URL.createObjectURL(blob);
		a.download = filename;
		a.click();
		URL.revokeObjectURL(a.href);
	}

	const downloadFiles = () => {
		const { single_row_csv, multi_row_csv, json } = convertMetricsToFiles(selectedDatasets);

		if (single_row_csv) {
			download(single_row_csv, 'datasets.csv');
		}

		if (multi_row_csv) {
			download(multi_row_csv, 'multiplexed-datasets.csv');
		}

		if (json) {
			download(json, 'datasets.json');
		}
	};
</script>

<FilteredList {limit} count={datasets.length}>
	{#snippet filters()}
		<FilterGroup label="Results">
			<Sort fields={sortFields} {orderBy} />
			<PageSize {limit} />
		</FilterGroup>
		<FilterGroup label="Dataset">
			<TagInput name={FormFieldName.Name} label="Name" />
			<MultiSelectFilter name={FormFieldName.SpecimenProjectId} label="Project" options={projectNames} />
			<DateRange
				from={FormFieldName.DeliveredFrom}
				to={FormFieldName.DeliveredTo}
				label="Date delivered"
			/>
		</FilterGroup>
		<FilterGroup label="Assay">
			<MultiSelectFilter name={FormFieldName.AssayName} label="Name" options={tenxAssays} />
			<MultiSelectFilter
				name={FormFieldName.AssayMultiplexingType}
				label="Multiplexing type"
				options={options.multiplexingTypes}
			/>
			<MultiSelectFilter
				name={FormFieldName.AssayLibraryType}
				label="Library type"
				options={options.libraryTypes}
			/>
		</FilterGroup>
		<FilterGroup label="Specimen">
			<TagInput name={FormFieldName.SpecimenName} label="Name" />
			<MultiSelectFilter name={FormFieldName.SpecimenType} label="Type" options={options.specimenTypes} />
			<MultiSelectFilter name={FormFieldName.SpecimenSpecies} label="Species" options={options.species} />
			<MultiSelectFilter
				name={FormFieldName.SpecimenEmbeddedIn}
				label="Embedded in"
				options={options.embeddingMatrices}
			/>
			<MultiSelectFilter
				name={FormFieldName.SpecimenFixative}
				label="Fixative"
				options={options.fixatives}
			/>
			<MultiSelectFilter
				name={FormFieldName.SpecimenThermalPreservationMethod}
				label="Thermal preservation method"
				options={options.thermalPreservationMethods}
			/>
			<DateRange
				from={FormFieldName.SpecimenReceivedFrom}
				to={FormFieldName.SpecimenReceivedTo}
				label="Date received"
			/>
		</FilterGroup>
	{/snippet}

	{#if datasets.length}
		<header class="cluster">
			<span>{datasetCount(datasets.length)}</span>
			<SelectionToolbar {selected} ids={datasets.map((ds) => ds.id)} />
			{#if selectedDatasets.length}
				<button class="button" onclick={downloadFiles}
					>Download metrics for {datasetCount(selectedDatasets.length)}</button
				>
			{/if}
		</header>

		<ul class="divided">
			{#each datasets as ds (ds.id)}
				{@const { id } = ds}
				<!-- All specimens in a dataset share the same species and project -->
				{@const { species, project_id } = ds.specimens[0]}
				{@const project = projects.get(project_id)}
				<ListRow
					title={ds.name}
					href={resolve('/(app)/chromium-datasets/[id]', { id })}
					subtitle={project?.name}
					subtitleHref={project && resolve('/(app)/projects/[id]', { id: project_id })}
					bind:checked={() => selected.has(id), (checked) => (checked ? selected.add(id) : selected.delete(id))}
				>
					<div class="cluster">
						{#each ds.specimens.slice(0, SPECIMEN_LIMIT) as s (s.id)}
							<a class="link-arrow" href={resolve('/(app)/specimens/[id]', { id: s.id })}
								>{s.name}</a
							>
						{/each}
						{#if ds.specimens.length > SPECIMEN_LIMIT}
							<span class="muted">+{ds.specimens.length - SPECIMEN_LIMIT} more</span>
						{/if}
					</div>
					<div class="muted">
						{formatAssayWithMultiplexing(ds.assay)} · <i>{formatSpecies(species)}</i> · Delivered
						<Time datetime={ds.delivered_at} />
					</div>
				</ListRow>
			{/each}
		</ul>
	{:else}
		<p>No datasets match these filters.</p>
	{/if}
</FilteredList>
