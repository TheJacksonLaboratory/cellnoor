import {
	blockEmbeddingMatrixValues,
	libraryTypeValues,
	sampleMultiplexingValues,
	speciesValues,
	specimenTypeValues
} from 'cellnoor-client/cellnoor-types.js';
import type { Fixative, ThermalPreservationMethod } from 'cellnoor-client/cellnoor-types.ts';
import {
	formatEmbeddingMatrix,
	formatEnum,
	formatFixative,
	formatSampleMultiplexing,
	formatSpecies
} from '../format.ts';

function toDisplayMap<T extends string>(
	values: readonly T[],
	toText: (value: T) => string = formatEnum
) {
	return new Map(values.map((v) => [v, toText(v)]));
}

export const multiplexingTypes = toDisplayMap(sampleMultiplexingValues, formatSampleMultiplexing);

export const libraryTypes = toDisplayMap(libraryTypeValues);

export const specimenTypes = toDisplayMap(specimenTypeValues);

export const species = toDisplayMap(speciesValues, formatSpecies);

export const fixatives = toDisplayMap(
	['dithiobis_succinimidylpropionate', 'formaldehyde_derivative'] satisfies Fixative[],
	formatFixative
);

export const embeddingMatrices = toDisplayMap(blockEmbeddingMatrixValues, formatEmbeddingMatrix);

export const thermalPreservationMethods = toDisplayMap([
	'controlled_rate_freezing',
	'flash_freezing'
] satisfies ThermalPreservationMethod[]);
