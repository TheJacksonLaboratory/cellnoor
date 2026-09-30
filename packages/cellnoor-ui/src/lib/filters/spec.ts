type DateComparison = { gte: string } | { lte: string };

export type Filter<P> = (params: URLSearchParams, name: string) => P[];

export type FilterSpec<P> = Record<string, Filter<P>>;

export function dateRangeParams(name: string) {
	return { from: `${name}_from`, to: `${name}_to` };
}

export function filterBuilders<P>() {
	return {
		values(toPredicate: (values: string[]) => P): Filter<P> {
			return (params, name) => [toPredicate(params.getAll(name))];
		},

		dateRange(toPredicate: (comparison: DateComparison) => P): Filter<P> {
			return (params, name) => {
				const { from, to } = dateRangeParams(name);
				const fromDate = params.get(from);
				const toDate = params.get(to);

				const predicates: P[] = [];
				if (fromDate) predicates.push(toPredicate({ gte: `${fromDate}T00Z` }));
				if (toDate) predicates.push(toPredicate({ lte: `${toDate}T23:59:59Z` }));
				return predicates;
			};
		}
	};
}

export function toPredicates<P>(spec: FilterSpec<P>, params: URLSearchParams) {
	return Object.entries(spec).flatMap(([name, filter]) => filter(params, name));
}

export function filterNames<S extends FilterSpec<unknown>>(spec: S) {
	return Object.fromEntries(Object.keys(spec).map((name) => [name, name])) as {
		[K in keyof S & string]: K;
	};
}
