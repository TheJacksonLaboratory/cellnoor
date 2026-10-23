export type Parser<P> = (values: string[]) => P | P[];

export function toPredicates<P>(parsers: Record<string, Parser<P>>, params: URLSearchParams) {
	return Object.entries(parsers).flatMap(([name, parse]) => parse(params.getAll(name)));
}

function startOfDay(date: string) {
	return new Date(`${date}T00:00`).toISOString();
}

export function endOfDay(date: string) {
	return new Date(`${date}T23:59:59.999`).toISOString();
}

export function onOrAfter(dates: string[]) {
	return dates.filter(Boolean).map((date) => ({ gte: startOfDay(date) }));
}

export function onOrBefore(dates: string[]) {
	return dates.filter(Boolean).map((date) => ({ lte: endOfDay(date) }));
}

export function toOrderBy<F extends string>(
	params: URLSearchParams,
	defaultField: F,
	defaultDesc = true
) {
	const desc = params.get('desc');

	return {
		field: (params.get('sort') ?? defaultField) as F,
		desc: desc === null ? defaultDesc : desc !== 'false'
	};
}

export function toLimit(params: URLSearchParams) {
	const limit = params.get('limit') ?? '50';
	return limit === 'all' ? null : Number(limit);
}
