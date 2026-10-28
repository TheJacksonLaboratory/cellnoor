import { cellnoorClient, MUTABLE_DATA, unwrap } from '#lib/client.ts';
import { toLimit, toOrderBy, toPredicates } from '#lib/filters/utils.ts';
import type { ProjectField, ProjectPredicate } from 'cellnoor-client/cellnoor-types.ts';
import { projectFilterFieldParsers } from './field-parsers.ts';

export async function load({ url, parent, depends }) {
	depends(MUTABLE_DATA);

	const orderBy = toOrderBy<ProjectField>(url.searchParams, 'started_at');
	const limit = toLimit(url.searchParams);

	const projectList = await cellnoorClient
		.POST('/projects/search/detailed', {
			body: {
				filter: {
					all_of: toPredicates<ProjectPredicate>(projectFilterFieldParsers, url.searchParams)
				},
				order_by: orderBy,
				limit
			}
		})
		.then(unwrap);

	const { people } = await parent();

	const personNames = new Map(people.entries().map(([id, p]) => [id, p.name]));

	return { projectList, personNames, orderBy, limit };
}
