import { cellnoorClient, MUTABLE_DATA, unwrap } from '#lib/client.ts';
import { toLimit, toOrderBy, toPredicates } from '#lib/filters/utils.ts';
import type { ServiceField, ServicePredicate } from 'cellnoor-client/cellnoor-types.ts';
import { serviceFilterFieldParsers } from './field-parsers.ts';

export async function load({ url, parent, depends }) {
	depends(MUTABLE_DATA);

	const orderBy = toOrderBy<ServiceField>(url.searchParams, 'created_at');
	const limit = toLimit(url.searchParams);

	const serviceList = await cellnoorClient
		.POST('/services/search/detailed', {
			body: {
				filter: {
					all_of: toPredicates<ServicePredicate>(serviceFilterFieldParsers, url.searchParams)
				},
				order_by: orderBy,
				limit
			}
		})
		.then(unwrap);

	const { people } = await parent();

	const personNames = new Map(people.entries().map(([id, p]) => [id, p.name]));

	return { serviceList, personNames, orderBy, limit };
}
