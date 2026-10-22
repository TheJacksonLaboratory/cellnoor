import { cellnoorClient, unwrap } from '#lib/client.ts';
import { toLimit, toOrderBy, toPredicates } from '#lib/filters/utils.ts';
import type { PersonField, PersonPredicate } from 'cellnoor-client/cellnoor-types.ts';
import { personFilterFieldParsers } from './field-parsers.ts';

export async function load({ url }) {
	const orderBy = toOrderBy<PersonField>(url.searchParams, 'name', false);
	const limit = toLimit(url.searchParams);

	const [personList, institutionList] = await Promise.all([
		cellnoorClient
			.POST('/people/search/permissions', {
				body: {
					filter: {
						all_of: toPredicates<PersonPredicate>(personFilterFieldParsers, url.searchParams)
					},
					order_by: orderBy,
					limit
				}
			})
			.then(unwrap),
		cellnoorClient.GET('/institutions').then(unwrap)
	]);

	const institutionNames = new Map(institutionList.map((i) => [i.id, i.name]));

	return { personList, institutionNames, orderBy, limit };
}
