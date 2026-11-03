<script lang="ts">
	import { resolve } from '$app/paths';
	import SidebarLayout from '#lib/SidebarLayout.svelte';
	import FileExplorer from '#lib/files/FileExplorer.svelte';
	import FileViewer from '#lib/files/FileViewer.svelte';
	import type { FileLeaf } from '#lib/files/file-tree.ts';
	import SpecimenList from '#lib/SpecimenList.svelte';
	import Time from '#lib/Time.svelte';
	import { formatEnum, formatSampleMultiplexing } from '#lib/format.ts';

	let { data } = $props();

	// All specimens in a dataset share the same project
	const project = $derived(data.projects.get(data.dataset.specimens[0].project_id)!);

	// Chromium datasets always have library types
	const libraryTypes = $derived(data.dataset.assay.library_types!.map(formatEnum).join(', '));

	// Chromium datasets always have some kind of multiplexing
	const multiplexing = $derived(formatSampleMultiplexing(data.dataset.assay.sample_multiplexing!));

	const iLabRequestIds: string[] | null = $derived(
		// @ts-expect-error I honestly don't know why typescript hates this
		data.dataset.chromium_run.additional_data?.ilab_request_ids
	);

	let selected = $state<FileLeaf>();
</script>

<SidebarLayout>
	{#snippet sidebar()}
		{@const { dataset } = data}
		<FileExplorer files={dataset.files} bind:selected />
	{/snippet}

	{@const { dataset } = data}
	<header class="stack card card-body">
		<h1 class="title">{dataset.name}</h1>

		<div class="scroll-x">
			<table>
				<thead>
					<tr>
						<th scope="col">Project</th>
						<th scope="col">Delivered</th>
						<th scope="col">Chromium run</th>
						<th scope="col">Assay</th>
						<th scope="col">Library types</th>
						<th scope="col">Multiplexing</th>
						{#if iLabRequestIds}
							<th scope="col">iLab Request IDs</th>
						{/if}
					</tr>
				</thead>
				<tbody>
					<tr>
						<td
							><a class="link-arrow" href={resolve('/(app)/projects/[id]', { id: project.id })}
								>{project.name}</a
							></td
						>
						<td><Time datetime={dataset.delivered_at} /></td>
						<td><Time datetime={dataset.chromium_run.run_at}></Time></td>
						<td
							><a href={dataset.assay.protocol_url} rel="external" class="link-arrow"
								>{`${dataset.assay.name} (${dataset.assay.chemistry_version})`}</a
							></td
						>
						<td>{libraryTypes}</td>
						<td>{multiplexing}</td>
						{#if iLabRequestIds}
							<td>{iLabRequestIds.join(' | ')}</td>
						{/if}
					</tr>
				</tbody>
			</table>
		</div>
	</header>

	{#if selected}
		<FileViewer file={selected} onclose={() => (selected = undefined)} />
	{/if}

	<section class="stack card card-body">
		<h2 class="heading">Specimens</h2>
		<SpecimenList specimens={dataset.specimens} people={data.people} />
	</section>
</SidebarLayout>
