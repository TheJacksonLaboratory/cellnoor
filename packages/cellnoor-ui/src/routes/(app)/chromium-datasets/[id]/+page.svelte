<script lang="ts">
	import DetailsLayout from '#lib/DetailsLayout.svelte';
	import FileExplorer from '#lib/files/FileExplorer.svelte';
	import FileViewer from '#lib/files/FileViewer.svelte';
	import type { FileLeaf } from '#lib/files/file-tree.ts';
	import SpecimenList from '#lib/SpecimenList.svelte';
	import { DATE_FORMATTER, formatEnum, formatSampleMultiplexing } from '#lib/format.ts';

	let { data } = $props();

	const { dataset, projects, people } = $derived(data);

	// All specimens in a dataset share the same project
	const project = $derived(projects.get(dataset.specimens[0].project_id));

	let selected = $state<FileLeaf>();
</script>

<DetailsLayout>
	{#snippet sidebar()}
		<FileExplorer files={dataset.files} bind:selected />
	{/snippet}

	<header>
		<h1 class="heading">{dataset.name}</h1>
		{#if project}
			<a class="subheading link-arrow" href={project.links.self}>{project.name}</a>
		{/if}
	</header>

	<dl class="properties">
		<dt>Delivered</dt>
		<dd>
			<time datetime={dataset.delivered_at}
				>{DATE_FORMATTER.format(new Date(dataset.delivered_at))}</time
			>
		</dd>
		<dt>Assay</dt>
		<dd>{dataset.assay.name}</dd>
		{#if dataset.assay.library_types?.length}
			<dt>Library types</dt>
			<dd>{dataset.assay.library_types.map(formatEnum).join(', ')}</dd>
		{/if}
		{#if dataset.assay.sample_multiplexing}
			<dt>Multiplexing</dt>
			<dd>{formatSampleMultiplexing(dataset.assay.sample_multiplexing)}</dd>
		{/if}
	</dl>

	<section class="stack">
		<h2 class="subheading">Specimens</h2>
		<SpecimenList specimens={dataset.specimens} {people} />
	</section>

	{#if selected}
		<FileViewer file={selected} />
	{/if}
</DetailsLayout>
