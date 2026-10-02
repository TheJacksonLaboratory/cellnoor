import type { TenxAssay } from 'cellnoor-client/cellnoor-types.ts';

export const DATE_FORMATTER = new Intl.DateTimeFormat('en-GB', {
	day: '2-digit',
	month: 'long',
	year: 'numeric'
});

export function formatSpecies(species: string) {
	return species.charAt(0).toUpperCase() + species.slice(1).replaceAll('_', ' ');
}

export function formatAssayWithMultiplexing({ name, sample_multiplexing }: TenxAssay) {
	if (!sample_multiplexing) {
		return name;
	}

	const renamed =
		sample_multiplexing === 'on_chip_multiplexing'
			? 'OCM'
			: sample_multiplexing.replaceAll('_', ' ');

	return `${name} (${renamed})`;
}
