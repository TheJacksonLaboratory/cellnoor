import { cellnoorClient, unwrap } from '#lib/client.ts';
import { toPredicates } from '#lib/filters/spec.ts';
import {
	blockEmbeddingMatrixValues,
	speciesValues,
	specimenTypeValues
} from 'cellnoor-client/cellnoor-types.js';
import { datasetFilters } from './filters.ts';
import { SelectableData } from '#lib/selectable-data.svelte.ts';

export async function load({ url, parent }) {
	const datasetList = await cellnoorClient
		.POST('/chromium-datasets/search/detailed', {
			body: { filter: { all_of: toPredicates(datasetFilters, url.searchParams) }, limit: 100 }
		})
		.then(unwrap);

	const { projects, tenxAssays } = await parent();

	const projectNames = new Map(projects.entries().map(([id, p]) => [id, p.name]));

	const datasets = new SelectableData(datasetList, (ds) => ds.id);

	const dsp = 'dithiobis_succinimidylpropionate';
	const toFixativeDisplayText = (fixative: string) =>
		({ [dsp]: 'DSP' })[fixative] ?? toDisplayText(fixative);

	const toSpeciesDisplayText = (species: string) =>
		toDisplayText(species.charAt(0).toUpperCase() + species.slice(1));

	const toEmbeddingDisplayText = (em: string) =>
		({
			optimal_cutting_temperature_compound: 'OCT',
			carboxymethyl_cellulose: 'CMC'
		})[em] ?? toDisplayText(em);

	return {
		datasets,
		projects,
		projectNames,
		tenxAssays,
		specimenTypes: toDisplayMap(specimenTypeValues),
		species: toDisplayMap(speciesValues, toSpeciesDisplayText),
		fixatives: toDisplayMap([dsp, 'formaldehyde_derivative'], toFixativeDisplayText),
		embeddingMatrices: toDisplayMap(blockEmbeddingMatrixValues, toEmbeddingDisplayText),
		thermalPreservationMethods: toDisplayMap(['controlled_rate_freezing', 'flash_freezing'])
	};
}

function toDisplayMap(strings: readonly string[], toText?: (s: string) => string) {
	const transform = toText ?? toDisplayText;

	return new Map(strings.map((s) => [s, transform(s)]));
}

function toDisplayText(s: string) {
	return s.replaceAll('_', ' ');
}
