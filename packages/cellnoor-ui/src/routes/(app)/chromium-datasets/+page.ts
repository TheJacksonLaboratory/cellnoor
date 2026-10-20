import { cellnoorClient, unwrap } from '#lib/client.ts';
import type { ChromiumDatasetPredicate } from 'cellnoor-client/cellnoor-types.ts';
import { toPredicates } from '#lib/filters/spec.ts';
import { datasetFilterFieldParsers } from './field-parsers.ts';
import { SelectableData } from '#lib/selectable-data.svelte.ts';

export async function load({ url, parent }) {
	const datasetList = await cellnoorClient
		.POST('/chromium-datasets/search/detailed', {
			body: {
				filter: {
					all_of: toPredicates<ChromiumDatasetPredicate>(
						datasetFilterFieldParsers,
						url.searchParams
					)
				},
				limit: 100
			}
		})
		.then(unwrap);

	const { projects } = await parent();

	const projectNames = new Map(projects.entries().map(([id, p]) => [id, p.name]));

	const datasets = new SelectableData(datasetList, (ds) => ds.id);

	return { datasets, projectNames };
}
