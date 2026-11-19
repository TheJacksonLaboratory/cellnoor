<script lang="ts">
	import { resolve } from '$app/paths';
	import type { ChromiumDatasetDetailed } from 'cellnoor-client/cellnoor-types.ts';

	let { keyMetrics }: { keyMetrics: ChromiumDatasetDetailed['key_metrics'] } = $props();
</script>

{#each Object.entries(keyMetrics) as [libraryType, specimens] (libraryType)}
	{#if specimens?.length}
		{@const metricNames = Object.keys(specimens[0].renamed_metrics)}
		<section class="stack card card-body">
			<h2 class="heading">{libraryType} metrics</h2>
			<div class="scroll-x">
				<table>
					<thead>
						<tr>
							<th scope="col">Specimen</th>
							{#each metricNames as name (name)}
								<th scope="col">{name}</th>
							{/each}
						</tr>
					</thead>
					<tbody>
						{#each specimens as s, i (i)}
							<tr>
								<td
									><a
										class="link-arrow"
										href={resolve('/(app)/specimens/[id]', { id: s.specimen_id })}
										>{s.specimen_name}</a
									></td
								>
								{#each metricNames as name (name)}
									<td>{s.renamed_metrics[name]}</td>
								{/each}
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</section>
	{/if}
{/each}
