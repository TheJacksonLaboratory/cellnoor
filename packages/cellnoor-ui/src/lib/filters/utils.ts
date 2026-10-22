export type Parser<P> = (values: string[]) => P | P[];

export function toPredicates<P>(parsers: Record<string, Parser<P>>, params: URLSearchParams) {
	return Object.entries(parsers).flatMap(([name, parse]) => parse(params.getAll(name)));
}

export function onOrAfter(dates: string[]) {
	return dates.filter(Boolean).map((date) => ({ gte: `${date}T00Z` }));
}

export function onOrBefore(dates: string[]) {
	return dates.filter(Boolean).map((date) => ({ lte: `${date}T23:59:59Z` }));
}

export function toOrderBy<F extends string>(params: URLSearchParams, defaultField: F) {
	return {
		field: (params.get('sort') ?? defaultField) as F,
		desc: params.get('desc') !== 'false'
	};
}

export function toLimit(params: URLSearchParams) {
	const limit = params.get('limit') ?? '50';
	return limit === 'all' ? null : Number(limit);
}
