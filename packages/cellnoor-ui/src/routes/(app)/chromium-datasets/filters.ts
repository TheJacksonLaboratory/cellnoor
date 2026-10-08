import { filterBuilders, filterNames } from '#lib/filters/spec.ts';
import type {
	BlockEmbeddingMatrix,
	ChromiumDatasetPredicate,
	Fixative,
	LibraryType,
	SampleMultiplexing,
	Species,
	SpecimenType,
	ThermalPreservationMethod
} from 'cellnoor-client/cellnoor-types.ts';

const { values, dateRange } = filterBuilders<ChromiumDatasetPredicate>();

export const datasetFilters = {
	name: values((v) => ({ name: { trgm_any_unless_empty: v } })),
	'specimen.project_id': values((v) => ({ specimen: { project_id: { in_unless_empty: v } } })),
	'assay.name': values((v) => ({ tenx_assay: { name: { in_unless_empty: v } } })),
	'assay.multiplexing_type': values((v) => ({
		tenx_assay: { sample_multiplexing: { in_unless_empty: v as SampleMultiplexing[] } }
	})),
	'assay.library_type': values((v) => ({
		tenx_assay: { library_types: { overlaps: v as LibraryType[] } }
	})),
	delivered: dateRange((exp) => ({ delivered_at: exp })),
	'specimen.name': values((v) => ({ specimen: { name: { trgm_any_unless_empty: v } } })),
	'specimen.type': values((v) => ({
		specimen: { type: { in_unless_empty: v as SpecimenType[] } }
	})),
	'specimen.species': values((v) => ({
		specimen: { species: { in_unless_empty: v as Species[] } }
	})),
	'specimen.embedded_in': values((v) => ({
		specimen: { embedded_in: { in_unless_empty: v as BlockEmbeddingMatrix[] } }
	})),
	'specimen.fixative': values((v) => ({
		specimen: { fixative: { in_unless_empty: v as Fixative[] } }
	})),
	'specimen.thermal_preservation_method': values((v) => ({
		specimen: { thermal_preservation_method: { in_unless_empty: v as ThermalPreservationMethod[] } }
	})),
	'specimen.received': dateRange((exp) => ({ specimen: { received_at: exp } }))
};

export const datasetFilterNames = filterNames(datasetFilters);
