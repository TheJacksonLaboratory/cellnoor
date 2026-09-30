import { cellnoorClient, unwrap } from '#lib/client.ts';
import { toPredicates } from '#lib/filters/spec.ts';
import {
	blockEmbeddingMatrixValues,
	speciesValues,
	specimenTypeValues
} from 'cellnoor-client/cellnoor-types.js';
import { datasetFilters } from './filters.ts';

export async function load({ url, parent }) {
	const datasets = unwrap(
		await cellnoorClient.POST('/chromium-datasets/search/detailed', {
			body: { filter: { all_of: toPredicates(datasetFilters, url.searchParams) }, limit: 100 }
		})
	);

	const { projects, tenxAssays } = await parent();

	return {
		datasets,
		projects: toComboboxOptions(projects),
		projectsById: new Map(projects.map((p) => [p.id, p])),
		assays: [...new Set(tenxAssays.map((a) => a.name))].map(toSimpleComboboxOption),
		specimenTypes: specimenTypeValues.map(toSimpleComboboxOption),
		species: speciesValues.map(toSimpleComboboxOption),
		fixatives: ['dithiobis_succinimidylpropionate', 'formaldehyde_derivative'].map(
			toSimpleComboboxOption
		),
		embeddingMatrices: blockEmbeddingMatrixValues.map(toSimpleComboboxOption),
		thermalPreservationMethods: ['controlled_rate_freezing', 'flash_freezing'].map(
			toSimpleComboboxOption
		)
	};
}

function toComboboxOptions(items: { id: string; name: string }[]) {
	return items.map((i) => ({ value: i.id, label: i.name }));
}

function toSimpleComboboxOption(item: string) {
	return { value: item, label: item };
}
