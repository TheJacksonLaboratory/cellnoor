<script lang="ts">
	import { page } from '$app/state';
	import Browser from '#lib/Browser.svelte';
	import FilterGroup from '#lib/filters/FilterGroup.svelte';
	import MultiSelect from '#lib/filters/MultiSelect.svelte';
	import TagInput from '#lib/filters/TagInput.svelte';
	import DateRange from '#lib/filters/DateRange.svelte';
	import ListRow from '#lib/ListRow.svelte';
	import { DATE_FORMATTER, formatAssayWithMultiplexing, formatSpecies } from '#lib/format.ts';
	import SelectionToolbar from '#lib/SelectionToolbar.svelte';
	import { Selection } from '#lib/selection.svelte.ts';
	import { datasetFilterNames as names } from './filters.ts';

	let { data } = $props();

	const {
		datasets,
		projects,
		projectsById,
		assays,
		specimenTypes,
		species,
		fixatives,
		embeddingMatrices,
		thermalPreservationMethods
	} = $derived(data);

	const SPECIMEN_LIMIT = 8;

	const selection = new Selection();
</script>

<Browser>
	{#snippet filters()}
		<FilterGroup label="Dataset information">
			<TagInput name={names.name} label="Name" />
			<MultiSelect name={names['specimen.project_id']} label="Project" options={projects} />
			<MultiSelect name={names['assay.name']} label="Assay" options={assays} />
			<DateRange name={names.delivered} label="Date delivered" />
		</FilterGroup>
		<FilterGroup label="Specimen information">
			<TagInput name={names['specimen.name']} label="Name" />
			<MultiSelect name={names['specimen.type']} label="Type" options={specimenTypes} />
			<MultiSelect name={names['specimen.species']} label="Species" options={species} />
			<MultiSelect
				name={names['specimen.embedded_in']}
				label="Embedded in"
				options={embeddingMatrices}
			/>
			<MultiSelect name={names['specimen.fixative']} label="Fixative" options={fixatives} />
			<MultiSelect
				name={names['specimen.thermal_preservation_method']}
				label="Thermal preservation method"
				options={thermalPreservationMethods}
			/>
			<DateRange name={names['specimen.received']} label="Date received" />
		</FilterGroup>
	{/snippet}

	<header class="cluster">
		<h1 class="heading">Datasets</h1>
		<span>{datasets.length} {datasets.length === 1 ? 'dataset' : 'datasets'}</span>
	</header>

	{#if datasets.length}
		<SelectionToolbar {selection} ids={datasets.map((d) => d.id)} />

		<ul class="divided">
			{#each datasets as d (d.id)}
				<!-- All specimens in a dataset share the same species and project -->
				{@const { species, project_id } = d.specimens[0]}
				{@const project = projectsById.get(project_id)}
				<ListRow
					title={d.name}
					href={d.links.self}
					subtitle={project?.name}
					subtitleHref={project?.links.self}
					bind:checked={selection.checked[d.id]}
				>
					<div class="cluster">
						{#each d.specimens.slice(0, SPECIMEN_LIMIT) as s (s.id)}
							<a class="link-arrow" href={s.links.self}>{s.name}</a>
						{/each}
						{#if d.specimens.length > SPECIMEN_LIMIT}
							<span class="muted">+{d.specimens.length - SPECIMEN_LIMIT} more</span>
						{/if}
					</div>
					<div class="muted">
						{formatAssayWithMultiplexing(d.assay)} · <i>{formatSpecies(species)}</i> · Delivered
						<time datetime={d.delivered_at}>{DATE_FORMATTER.format(new Date(d.delivered_at))}</time>
					</div>
				</ListRow>
			{/each}
		</ul>
	{:else}
		<p>No datasets match these filters. <a href={page.url.pathname}>Clear all filters</a></p>
	{/if}
</Browser>

<style>
	header {
		justify-content: space-between;
	}
</style>
