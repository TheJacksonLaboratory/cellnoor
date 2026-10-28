import { cellnoorClient, MUTABLE_DATA, unwrap } from '#lib/client.ts';
import { toLimit, toOrderBy, toPredicates } from '#lib/filters/utils.ts';
import type { BotAccountField, BotAccountPredicate } from 'cellnoor-client/cellnoor-types.ts';
import { bot_accountFilterFieldParsers } from './field-parsers.ts';

export async function load({ url, parent, depends }) {
	depends(MUTABLE_DATA);

	const orderBy = toOrderBy<BotAccountField>(url.searchParams, 'created_at');
	const limit = toLimit(url.searchParams);

	const bot_accountList = await cellnoorClient
		.POST('/bot-accounts/search/detailed', {
			body: {
				filter: {
					all_of: toPredicates<BotAccountPredicate>(bot_accountFilterFieldParsers, url.searchParams)
				},
				order_by: orderBy,
				limit
			}
		})
		.then(unwrap);

	const { people } = await parent();

	const personNames = new Map(people.entries().map(([id, p]) => [id, p.name]));

	return { bot_accountList, personNames, orderBy, limit };
}
