import { cellnoorClient, unwrap } from '#lib/client.ts';
import type {
	ChromiumDatasetField,
	ChromiumDatasetPredicate
} from 'cellnoor-client/cellnoor-types.ts';
import { toLimit, toOrderBy, toPredicates } from '#lib/filters/utils.ts';
import { datasetFilterFieldParsers } from './field-parsers.ts';
import { SelectableData } from '#lib/selectable-data.svelte.ts';

export async function load({ url, parent }) {
	const orderBy = toOrderBy<ChromiumDatasetField>(url.searchParams, 'delivered_at');
	const limit = toLimit(url.searchParams);

	const datasetList = await cellnoorClient
		.POST('/chromium-datasets/search/detailed', {
			body: {
				filter: {
					all_of: toPredicates<ChromiumDatasetPredicate>(
						datasetFilterFieldParsers,
						url.searchParams
					)
				},
				order_by: orderBy,
				limit
			}
		})
		.then(unwrap);

	const { projects } = await parent();

	const projectNames = new Map(projects.entries().map(([id, p]) => [id, p.name]));

	const datasets = new SelectableData(datasetList, (ds) => ds.id);

	return { datasets, projectNames, orderBy, limit };
}
