<script lang="ts">
	import Browser from '#lib/Browser.svelte';
	import FilterGroup from '#lib/filters/FilterGroup.svelte';
	import MultiSelect from '#lib/filters/MultiSelect.svelte';
	import TagInput from '#lib/filters/TagInput.svelte';
	import DateRange from '#lib/filters/DateRange.svelte';
	import ListRow from '#lib/ListRow.svelte';
	import { DATE_FORMATTER, formatAssayWithMultiplexing, formatSpecies } from '#lib/format.ts';
	import SelectionToolbar from '#lib/SelectionToolbar.svelte';
	import { datasetFilterNames as names } from './filters.ts';

	let { data } = $props();

	const {
		datasets,
		projects,
		projectNames,
		tenxAssays,
		specimenTypes,
		species,
		fixatives,
		embeddingMatrices,
		thermalPreservationMethods
	} = $derived(data);

	const SPECIMEN_LIMIT = 8;
</script>

<Browser>
	{#snippet filters()}
		<FilterGroup label="Dataset information">
			<TagInput name={names.name} label="Name" />
			<MultiSelect name={names['specimen.project_id']} label="Project" options={projectNames} />
			<MultiSelect name={names['assay.name']} label="Assay" options={tenxAssays} />
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
