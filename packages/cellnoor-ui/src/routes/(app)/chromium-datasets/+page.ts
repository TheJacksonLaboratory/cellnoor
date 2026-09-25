import { cellnoorClient, unwrap } from '#lib/client.ts';
import {
	blockEmbeddingMatrixValues,
	controlledRateFreezingValues,
	flashFreezingValues,
	speciesValues,
	specimenTypeValues,
	type ChromiumDatasetPredicateFilter as Filter
} from 'cellnoor-client/cellnoor-types';

export async function load({ url }) {
	const q = url.searchParams;

	const oneOf = <T extends string>(key: string, values: readonly T[]) =>
		q.getAll(key).filter((v): v is T => values.some((value) => value === v));

	const nonEmpty = <T>(values: T[], toFilter: (values: T[]) => Filter) =>
		values.length > 0 ? toFilter(values) : undefined;

	const toTimestamp = (key: string, toFilter: (timestamp: string) => Filter) => {
		const value = q.get(key);
		return value ? toFilter(new Date(`${value}T00:00`).toISOString()) : undefined;
	};

	const all_of = [
		nonEmpty(q.getAll('project'), (v) => ({ specimen: { project_id: { in: v } } })),
		nonEmpty(q.getAll('specimen_name'), (v) => ({ specimen: { name: { trgm_any: v } } })),
		nonEmpty(oneOf('species', speciesValues), (v) => ({ specimen: { species: { in: v } } })),
		toTimestamp('received_from', (t) => ({ specimen: { received_at: { gte: t } } })),
		toTimestamp('received_to', (t) => ({ specimen: { received_at: { lte: t } } })),
		nonEmpty(oneOf('specimen_type', specimenTypeValues), (v) => ({
			specimen: { type: { in: v } }
		})),
		nonEmpty(oneOf('embedded_in', blockEmbeddingMatrixValues), (v) => ({
			specimen: { embedded_in: { in: v } }
		})),
		nonEmpty(oneOf('thermal_preservation', [...controlledRateFreezingValues, ...flashFreezingValues]), (v) => ({
			specimen: { thermal_preservation_method: { in: v } }
		})),
		nonEmpty(q.getAll('assay'), (v) => ({ tenx_assay: { name: { in: v } } })),
		nonEmpty(q.getAll('name'), (v) => ({ name: { trgm_any: v } })),
		toTimestamp('delivered_from', (t) => ({ delivered_at: { gte: t } })),
		toTimestamp('delivered_to', (t) => ({ delivered_at: { lte: t } }))
	].filter((f) => f !== undefined);

	const datasets = unwrap(
		await cellnoorClient.POST('/chromium-datasets/search/detailed', {
			body: { filter: all_of.length > 0 ? { all_of } : undefined }
		})
	);

	return { datasets, datasetIds: datasets.map((ds) => ds.id) };
}
