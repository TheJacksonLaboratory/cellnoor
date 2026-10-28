import type { SampleMultiplexing, TenxAssay } from 'cellnoor-client/cellnoor-types.ts';

export const DATE_FORMATTER = new Intl.DateTimeFormat('en-GB', {
	day: '2-digit',
	month: 'long',
	year: 'numeric'
});

export function formatEnum(s: string) {
	return s.replaceAll('_', ' ');
}

export function formatSpecies(species: string) {
	return species.charAt(0).toUpperCase() + formatEnum(species.slice(1));
}

export function formatFixative(fixative: string) {
	return fixative === 'dithiobis_succinimidylpropionate' ? 'DSP' : formatEnum(fixative);
}

export function formatEmbeddingMatrix(em: string) {
	switch (em) {
		case 'optimal_cutting_temperature_compound':
			return 'OCT';
		case 'carboxymethyl_cellulose':
			return 'CMC';
		default:
			return formatEnum(em);
	}
}

export function formatSampleMultiplexing(sampleMultiplexing: SampleMultiplexing) {
	return sampleMultiplexing === 'on_chip_multiplexing' ? 'OCM' : formatEnum(sampleMultiplexing);
}

export function formatAssayWithMultiplexing({ name, sample_multiplexing }: TenxAssay) {
	if (!sample_multiplexing) {
		return name;
	}

	return `${name} (${formatSampleMultiplexing(sample_multiplexing)})`;
}
