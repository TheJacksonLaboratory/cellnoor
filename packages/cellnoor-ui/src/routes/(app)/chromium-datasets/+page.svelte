<script lang="ts">
	import Browser from '#lib/Browser.svelte';
	import FilterGroup from '#lib/filters/FilterGroup.svelte';
	import MultiSelect from '#lib/filters/MultiSelect.svelte';
	import TagInput from '#lib/filters/TagInput.svelte';
	import DateRange from '#lib/filters/DateRange.svelte';
	import * as options from '#lib/filters/options.ts';
	import ListRow from '#lib/ListRow.svelte';
	import Time from '#lib/Time.svelte';
	import { formatAssayWithMultiplexing, formatSpecies } from '#lib/format.ts';
	import SelectionToolbar from '#lib/SelectionToolbar.svelte';
	import { FormFieldName } from './field-parsers.ts';
	import { convertMetricsToFiles } from './convert-metrics.ts';

	let { data } = $props();

	const { datasets, projects, projectNames, tenxAssays } = $derived(data);

	const SPECIMEN_LIMIT = 8;

	const datasetCount = (n: number) => `${n} ${n === 1 ? 'dataset' : 'datasets'}`;

	function download(blob: Blob, filename: string) {
		const a = document.createElement('a');
		a.href = URL.createObjectURL(blob);
		a.download = filename;
		a.click();
	}

	const downloadFiles = () => {
		const { single_row_csv, multi_row_csv, json } = convertMetricsToFiles(datasets.selectedItems);

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

<Browser>
	{#snippet filters()}
		<FilterGroup label="Dataset">
			<TagInput name={FormFieldName.Name} label="Name" />
			<MultiSelect name={FormFieldName.SpecimenProjectId} label="Project" options={projectNames} />
			<DateRange
				from={FormFieldName.DeliveredFrom}
				to={FormFieldName.DeliveredTo}
				label="Date delivered"
			/>
		</FilterGroup>
		<FilterGroup label="Assay">
			<MultiSelect name={FormFieldName.AssayName} label="Name" options={tenxAssays} />
			<MultiSelect
				name={FormFieldName.AssayMultiplexingType}
				label="Multiplexing type"
				options={options.multiplexingTypes}
			/>
			<MultiSelect
				name={FormFieldName.AssayLibraryType}
				label="Library type"
				options={options.libraryTypes}
			/>
		</FilterGroup>
		<FilterGroup label="Specimen">
			<TagInput name={FormFieldName.SpecimenName} label="Name" />
			<MultiSelect name={FormFieldName.SpecimenType} label="Type" options={options.specimenTypes} />
			<MultiSelect name={FormFieldName.SpecimenSpecies} label="Species" options={options.species} />
			<MultiSelect
				name={FormFieldName.SpecimenEmbeddedIn}
				label="Embedded in"
				options={options.embeddingMatrices}
			/>
			<MultiSelect
				name={FormFieldName.SpecimenFixative}
				label="Fixative"
				options={options.fixatives}
			/>
			<MultiSelect
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

	{#if datasets.size}
		<header class="cluster">
			<span>{datasetCount(datasets.size)}</span>
			<SelectionToolbar selection={datasets} />
			{#if datasets.nSelected}
				<button class="button" onclick={downloadFiles}
					>Download metrics for {datasetCount(datasets.nSelected)}</button
				>
			{/if}
		</header>

		<ul class="divided">
			{#each datasets.entries() as [id, ds] (id)}
				<!-- All specimens in a dataset share the same species and project -->
				{@const { species, project_id } = ds.specimens[0]}
				{@const project = projects.get(project_id)}
				<ListRow
					title={ds.name}
					href={ds.links.self}
					subtitle={project?.name}
					subtitleHref={project?.links.self}
					bind:checked={datasets.isSelectedGetter(id), datasets.isSelectedSetter(id)}
				>
					<div class="cluster">
						{#each ds.specimens.slice(0, SPECIMEN_LIMIT) as s (s.id)}
							<a class="link-arrow" href={s.links.self}>{s.name}</a>
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
</Browser>
