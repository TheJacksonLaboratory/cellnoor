import { cellnoorClient, unwrap } from '#lib/client.ts';
import {
	blockEmbeddingMatrixValues,
	libraryTypeValues,
	multiplexingTagTypeValues,
	speciesValues,
	specimenTypeValues
} from 'cellnoor-client/cellnoor-types.js';
import type { ChromiumDatasetPredicate } from 'cellnoor-client/cellnoor-types.ts';
import { toPredicates } from '#lib/filters/spec.ts';
import { datasetFilterFieldParsers } from './field-parsers.ts';
import { SelectableData } from '#lib/selectable-data.svelte.ts';
import { formatEmbeddingMatrix, formatEnum, formatFixative, formatSpecies } from '#lib/format.ts';

export async function load({ url, parent }) {
	const datasetList = await cellnoorClient
		.POST('/chromium-datasets/search/detailed', {
			body: {
				filter: {
					all_of: toPredicates<ChromiumDatasetPredicate>(
						datasetFilterFieldParsers,
						url.searchParams
					)
				},
				limit: 100
			}
		})
		.then(unwrap);

	const { projects } = await parent();

	const projectNames = new Map(projects.entries().map(([id, p]) => [id, p.name]));

	const datasets = new SelectableData(datasetList, (ds) => ds.id);

	const dsp = 'dithiobis_succinimidylpropionate';

	return {
		datasets,
		projects,
		projectNames,
		multiplexingTypes: toDisplayMap(multiplexingTagTypeValues),
		libraryTypes: toDisplayMap(libraryTypeValues),
		specimenTypes: toDisplayMap(specimenTypeValues),
		species: toDisplayMap(speciesValues, formatSpecies),
		fixatives: toDisplayMap([dsp, 'formaldehyde_derivative'], formatFixative),
		embeddingMatrices: toDisplayMap(blockEmbeddingMatrixValues, formatEmbeddingMatrix),
		thermalPreservationMethods: toDisplayMap(['controlled_rate_freezing', 'flash_freezing'])
	};
}

function toDisplayMap(strings: readonly string[], toText?: (s: string) => string) {
	const transform = toText ?? formatEnum;

	return new Map(strings.map((s) => [s, transform(s)]));
}
