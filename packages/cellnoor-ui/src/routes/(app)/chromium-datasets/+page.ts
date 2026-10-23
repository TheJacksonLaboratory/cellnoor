import { cellnoorClient, unwrap } from '#lib/client.ts';
import type {
	ChromiumDatasetField,
	ChromiumDatasetPredicate
} from 'cellnoor-client/cellnoor-types.ts';
import { toLimit, toOrderBy, toPredicates } from '#lib/filters/utils.ts';
import { datasetFilterFieldParsers } from './field-parsers.ts';
import { SvelteSet } from 'svelte/reactivity';

export async function load({ url, parent }) {
	const orderBy = toOrderBy<ChromiumDatasetField>(url.searchParams, 'delivered_at');
	const limit = toLimit(url.searchParams);

	const datasets = await cellnoorClient
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

	return { datasets, selected: new SvelteSet<string>(), projectNames, orderBy, limit };
}
