import { cellnoorClient, unwrap } from '#lib/client.ts';
import {
	blockEmbeddingMatrixValues,
	speciesValues,
	specimenTypeValues,
	type BlockEmbeddingMatrix,
	type ChromiumDatasetPredicate,
	type Fixative,
	type Species,
	type SpecimenType,
	type ThermalPreservationMethod
} from 'cellnoor-client/cellnoor-types.js';

export async function load({ url, parent }) {
	const datasets = await getDatasets(url.searchParams);

	const { projects, tenxAssays } = await parent();

	return {
		datasets,
		projects: toComboboxOptions(projects),
		assays: [...new Set(tenxAssays.map((a) => a.name).map(toSimpleComboboxOption))],
		specimenTypes: specimenTypeValues.map(toSimpleComboboxOption),
		species: speciesValues.map(toSimpleComboboxOption),
		fixatives: ['dithiobis_succinimidyl_propionate', 'formaldehyde_derivative'].map(
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
		{ name: { trgm_any_unless_empty: q.getAll('name') } },
		{ tenx_assay: { name: { in_unless_empty: q.getAll('assay.name') } } },
		{ specimen: { name: { trgm_any_unless_empty: q.getAll('specimen.name') } } },
		{
			specimen: { type: { in_unless_empty: q.getAll('specimen.type') as SpecimenType[] } }
		},
		{
			specimen: { species: { in_unless_empty: q.getAll('specimen.species') as Species[] } }
		},
		{
			specimen: { fixative: { in_unless_empty: q.getAll('specimen.fixative') as Fixative[] } }
		},
		{
			specimen: {
				embedded_in: { in_unless_empty: q.getAll('specimen.embedded_in') as BlockEmbeddingMatrix[] }
			}
		},
		{
			specimen: {
				thermal_preservation_method: {
					in_unless_empty: q.getAll('specimen.species') as ThermalPreservationMethod[]
				}
			}
		},
		{
			specimen: { project_id: { in_unless_empty: q.getAll('specimen.project_id') } }
		}
	];

	const fieldNamesAndToFilters: [
		string,
		(exp: { gte: string } | { lte: string }) => ChromiumDatasetPredicate
	][] = [
		['delivered_from', (exp) => ({ delivered_at: exp })],
		[
			'delivered_after',
			(exp) => ({
				delivered_at: exp
			})
		],
		[
			'specimen.received_before',
			(exp) => ({
				specimen: {
					received_at: exp
				}
			})
		],
		['specimen.received_after', (exp) => ({ specimen: { received_at: exp } })]
	];

	for (const [fieldName, toFilter] of fieldNamesAndToFilters) {
		const fromOrTo = fieldName.split('_').at(-1) as 'from' | 'to';

		const filter = dateStringToFilter(q.get(fieldName), fromOrTo, toFilter);

		if (filter) {
			all_of.push(filter);
		}
	}

	const apiQuery = {
		filter: { all_of },
		limit: 100
	};

	const datasets = unwrap(
		await cellnoorClient.POST('/chromium-datasets/search/detailed', { body: apiQuery })
	);

	return datasets;
}

function toComboboxOptions(items: { id: string; name: string }[]) {
	return items.map((i) => ({ value: i.id, label: i.name }));
}

function toSimpleComboboxOption(item: string) {
	return { value: item, label: item };
}

function dateStringToFilter<T>(
	date: string | null,
	fromOrTo: 'from' | 'to',
	toFilter: (exp: { gte: string } | { lte: string }) => T
) {
	if (!date) {
		return null;
	}

	const exp = fromOrTo === 'from' ? { gte: `${date}T00Z` } : { lte: `${date}T23Z` };

	return toFilter(exp);
}
