import { onOrAfter, onOrBefore, type Parser } from '#lib/filters/spec.ts';
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

export enum FormFieldName {
	Name = 'name',
	SpecimenProjectId = 'specimen.project_id',
	AssayName = 'assay.name',
	AssayMultiplexingType = 'assay.multiplexing_type',
	AssayLibraryType = 'assay.library_type',
	DeliveredFrom = 'delivered_from',
	DeliveredTo = 'delivered_to',
	SpecimenName = 'specimen.name',
	SpecimenType = 'specimen.type',
	SpecimenSpecies = 'specimen.species',
	SpecimenEmbeddedIn = 'specimen.embedded_in',
	SpecimenFixative = 'specimen.fixative',
	SpecimenThermalPreservationMethod = 'specimen.thermal_preservation_method',
	SpecimenReceivedFrom = 'specimen.received_from',
	SpecimenReceivedTo = 'specimen.received_to'
}

export const datasetFilterFieldParsers: Record<FormFieldName, Parser<ChromiumDatasetPredicate>> = {
	name: (v) => ({ name: { trgm_any_unless_empty: v } }),
	'specimen.project_id': (v) => ({ specimen: { project_id: { in_unless_empty: v } } }),
	'assay.name': (v) => ({ tenx_assay: { name: { in_unless_empty: v } } }),
	'assay.multiplexing_type': (v) => ({
		tenx_assay: { sample_multiplexing: { in_unless_empty: v as SampleMultiplexing[] } }
	}),
	'assay.library_type': (v) => ({
		tenx_assay: { library_types: { contains: v as LibraryType[] } }
	}),
	delivered_from: (v) => onOrAfter(v).map((exp) => ({ delivered_at: exp })),
	delivered_to: (v) => onOrBefore(v).map((exp) => ({ delivered_at: exp })),
	'specimen.name': (v) => ({ specimen: { name: { trgm_any_unless_empty: v } } }),
	'specimen.type': (v) => ({ specimen: { type: { in_unless_empty: v as SpecimenType[] } } }),
	'specimen.species': (v) => ({ specimen: { species: { in_unless_empty: v as Species[] } } }),
	'specimen.embedded_in': (v) => ({
		specimen: { embedded_in: { in_unless_empty: v as BlockEmbeddingMatrix[] } }
	}),
	'specimen.fixative': (v) => ({ specimen: { fixative: { in_unless_empty: v as Fixative[] } } }),
	'specimen.thermal_preservation_method': (v) => ({
		specimen: { thermal_preservation_method: { in_unless_empty: v as ThermalPreservationMethod[] } }
	}),
	'specimen.received_from': (v) => onOrAfter(v).map((exp) => ({ specimen: { received_at: exp } })),
	'specimen.received_to': (v) => onOrBefore(v).map((exp) => ({ specimen: { received_at: exp } }))
};
