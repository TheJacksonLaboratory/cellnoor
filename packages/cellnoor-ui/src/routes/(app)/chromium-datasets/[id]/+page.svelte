<script lang="ts">
	import SidebarLayout from '#lib/SidebarLayout.svelte';
	import FileExplorer from '#lib/files/FileExplorer.svelte';
	import FileViewer from '#lib/files/FileViewer.svelte';
	import type { FileLeaf } from '#lib/files/file-tree.ts';
	import SpecimenList from '#lib/SpecimenList.svelte';
	import Time from '#lib/Time.svelte';
	import { formatEnum, formatSampleMultiplexing } from '#lib/format.ts';

	let { data } = $props();

	const { dataset, projects, people } = $derived(data);

	// All specimens in a dataset share the same project
	const project = $derived(projects.get(dataset.specimens[0].project_id));

	const libraryTypes = $derived(dataset.assay.library_types?.map(formatEnum).join(', '));

	const multiplexing = $derived(
		dataset.assay.sample_multiplexing && formatSampleMultiplexing(dataset.assay.sample_multiplexing)
	);

	let selected = $state<FileLeaf>();
</script>

<SidebarLayout>
	{#snippet sidebar()}
		<FileExplorer files={dataset.files} bind:selected />
	{/snippet}

	<header class="stack card card-body">
		<h1 class="title">{dataset.name}</h1>

		<div class="scroll-x">
			<table>
				<thead>
					<tr>
						{#if project}
							<th scope="col">Project</th>
						{/if}
						<th scope="col">Delivered</th>
						<th scope="col">Assay</th>
						{#if libraryTypes}
							<th scope="col">Library types</th>
						{/if}
						{#if multiplexing}
							<th scope="col">Multiplexing</th>
						{/if}
					</tr>
				</thead>
				<tbody>
					<tr>
						{#if project}
							<td><a class="link-arrow" href={project.links.self}>{project.name}</a></td>
						{/if}
						<td><Time datetime={dataset.delivered_at} /></td>
						<td>{dataset.assay.name}</td>
						{#if libraryTypes}
							<td>{libraryTypes}</td>
						{/if}
						{#if multiplexing}
							<td>{multiplexing}</td>
						{/if}
					</tr>
				</tbody>
			</table>
		</div>
	</header>

	{#if selected}
		<FileViewer file={selected} />
	{/if}

	<section class="stack card card-body">
		<h2 class="heading">Specimens</h2>
		<SpecimenList specimens={dataset.specimens} {people} />
	</section>
</SidebarLayout>
