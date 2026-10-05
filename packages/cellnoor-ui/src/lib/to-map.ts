export function toStringMap<T>(items: T[], toEntry: (item: T) => [string, string]) {
	return new Map(items.map(toEntry));
}
