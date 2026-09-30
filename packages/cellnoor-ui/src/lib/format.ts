export const DATE_FORMATTER = new Intl.DateTimeFormat('en-GB', {
	day: '2-digit',
	month: 'long',
	year: 'numeric'
});

export function formatSpecies(species: string) {
	return species.charAt(0).toUpperCase() + species.slice(1).replaceAll('_', ' ');
}
