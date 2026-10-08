<script lang="ts">
	import Browser from '#lib/Browser.svelte';
	import FilterGroup from '#lib/filters/FilterGroup.svelte';
	import MultiSelect from '#lib/filters/MultiSelect.svelte';
	import TagInput from '#lib/filters/TagInput.svelte';
	import DateRange from '#lib/filters/DateRange.svelte';
	import ListRow from '#lib/ListRow.svelte';
	import { DATE_FORMATTER, formatAssayWithMultiplexing, formatSpecies } from '#lib/format.ts';
	import SelectionToolbar from '#lib/SelectionToolbar.svelte';
	import { datasetFilterFieldParsers as fields } from './field-parsers.ts';
	import type { ChromiumDatasetDetailed } from 'cellnoor-client/cellnoor-types.ts';

	let { data } = $props();

	const {
		datasets,
		projects,
		projectNames,
		tenxAssays,
		multiplexingTypes,
		libraryTypes,
		specimenTypes,
		species,
		fixatives,
		embeddingMatrices,
		thermalPreservationMethods
	} = $derived(data);

	const SPECIMEN_LIMIT = 8;

	let firstSelectedDataset = $state<ChromiumDatasetDetailed>();

	const setFirstSelectedDataset = (id: string) => {
		if (firstSelectedDataset === undefined) {
			firstSelectedDataset = datasets.get(id)?.item;
		} else if (datasets.nSelected === 0) {
			firstSelectedDataset = undefined;
		}
	};

	// We assume that if two datasets "data" property have the same length, then they are compatible with one another
	const isCompatibleWithFirstDataset = (id: string) =>
		datasets.get(id)?.item.data.length === firstSelectedDataset?.data.length;
</script>

<Browser>
	{#snippet filters()}
		<FilterGroup label="Dataset">
			<TagInput {fields} name="name" label="Name" />
			<MultiSelect {fields} name="specimen.project_id" label="Project" options={projectNames} />
			<DateRange {fields} from="delivered_from" to="delivered_to" label="Date delivered" />
		</FilterGroup>
		<FilterGroup label="Assay">
			<MultiSelect {fields} name="assay.name" label="Name" options={tenxAssays} />
			<MultiSelect
				{fields}
				name="assay.multiplexing_type"
				label="Multiplexing Type"
				options={multiplexingTypes}
			></MultiSelect>
			<MultiSelect {fields} name="assay.library_type" label="Library Type" options={libraryTypes}
			></MultiSelect>
		</FilterGroup>
		<FilterGroup label="Specimen">
			<TagInput {fields} name="specimen.name" label="Name" />
			<MultiSelect {fields} name="specimen.type" label="Type" options={specimenTypes} />
			<MultiSelect {fields} name="specimen.species" label="Species" options={species} />
			<MultiSelect
				{fields}
				name="specimen.embedded_in"
				label="Embedded in"
				options={embeddingMatrices}
			/>
			<MultiSelect {fields} name="specimen.fixative" label="Fixative" options={fixatives} />
			<MultiSelect
				{fields}
				name="specimen.thermal_preservation_method"
				label="Thermal preservation method"
				options={thermalPreservationMethods}
			/>
			<DateRange
				{fields}
				from="specimen.received_from"
				to="specimen.received_to"
				label="Date received"
			/>
		</FilterGroup>
	{/snippet}

	{#if datasets.size}
		<header class="cluster">
			<span>{datasets.size} {datasets.size === 1 ? 'dataset' : 'datasets'}</span>
			<SelectionToolbar selection={datasets} />
		</header>

		<ul class="divided">
			{#each datasets.entries() as [id, ds] (id)}
				<!-- All specimens in a dataset share the same species and project -->
				{@const { species, project_id } = ds.item.specimens[0]}
				{@const project = projects.get(project_id)}
				<ListRow
					title={ds.item.name}
					href={ds.item.links.self}
					subtitle={project?.name}
					subtitleHref={project?.links.self}
					bind:checked={ds.selected}
					onCheck={() => setFirstSelectedDataset(id)}
					disabled={isCompatibleWithFirstDataset(id)}
				>
					<div class="cluster">
						{#each ds.item.specimens.slice(0, SPECIMEN_LIMIT) as s (s.id)}
							<a class="link-arrow" href={s.links.self}>{s.name}</a>
						{/each}
						{#if ds.item.specimens.length > SPECIMEN_LIMIT}
							<span class="muted">+{ds.item.specimens.length - SPECIMEN_LIMIT} more</span>
						{/if}
					</div>
					<div class="muted">
						{formatAssayWithMultiplexing(ds.item.assay)} · <i>{formatSpecies(species)}</i> ·
						Delivered
						<time datetime={ds.item.delivered_at}
							>{DATE_FORMATTER.format(new Date(ds.item.delivered_at))}</time
						>
					</div>
				</ListRow>
			{/each}
		</ul>
	{:else}
		<p>No datasets match these filters.</p>
	{/if}
</Browser>

<style>
	header {
		justify-content: space-between;
	}
</style>
