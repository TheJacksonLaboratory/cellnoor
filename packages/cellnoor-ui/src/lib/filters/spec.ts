type Parser<P> = (values: string[]) => P | P[];

export function toPredicates<P>(parsers: Record<string, Parser<P>>, params: URLSearchParams) {
	return Object.entries(parsers).flatMap(([name, parse]) => parse(params.getAll(name)));
}

export function onOrAfter(dates: string[]) {
	return dates.filter(Boolean).map((date) => ({ gte: `${date}T00Z` }));
}

export function onOrBefore(dates: string[]) {
	return dates.filter(Boolean).map((date) => ({ lte: `${date}T23:59:59Z` }));
}
