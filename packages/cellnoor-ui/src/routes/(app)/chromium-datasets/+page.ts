import { cellnoorClient, unwrap } from '#lib/client.ts';
import {
	blockEmbeddingMatrixValues,
	speciesValues,
	specimenTypeValues,
	type ChromiumDatasetPredicate,
	type Species,
	type SpecimenType
} from 'cellnoor-client/cellnoor-types.js';

export async function load({ url, parent }) {
	const datasets = await getDatasets(url.searchParams);

	const { projects, tenxAssays } = await parent();

	return {
		datasets,
		projects: toComboboxOptions(projects),
		assays: toComboboxOptions(tenxAssays),
		specimenTypes: specimenTypeValues.map(toSimpleComboboxOption),
		species: speciesValues.map(toSimpleComboboxOption),
		fixative: ['dithiobis_succinimidyl_propionate', 'formaldehyde_derivative'].map(
			toSimpleComboboxOption
		),
		embeddingMatrices: blockEmbeddingMatrixValues.map(toSimpleComboboxOption),
		thermalPreservationMethods: ['controlled_rate_freezing', 'flash_freezing'].map(
			toSimpleComboboxOption
		)
	};
}

async function getDatasets(q: URLSearchParams) {
	const all_of: ChromiumDatasetPredicate[] = [
		{ specimen: { name: { trgm_any_unless_empty: getQueryParam(q, 'specimen.name') } } },
		{
			specimen: { type: { in_unless_empty: getQueryParam(q, 'specimen.type') as SpecimenType[] } }
		},
		{
			specimen: { species: { in_unless_empty: getQueryParam(q, 'specimen.species') as Species[] } }
		}
	];

	const apiQuery = {
		filter: { all_of },
		limit: 100
	};

	const datasets = unwrap(
		await cellnoorClient.POST('/chromium-datasets/search/detailed', { body: apiQuery })
	);

	return datasets;
}

function getQueryParam(q: URLSearchParams, name: QueryParameter) {
	return q.getAll(name);
}

function toComboboxOptions(items: { id: string; name: string }[]) {
	return items.map((i) => ({ value: i.id, label: i.name }));
}

function toSimpleComboboxOption(item: string) {
	return { value: item, label: item };
}
