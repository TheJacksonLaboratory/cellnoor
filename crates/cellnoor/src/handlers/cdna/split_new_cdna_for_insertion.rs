use cellnoor_types::{
    cdna::{
        CdnaSimpleFields, NewCdnaRecord,
        creation::{CdnaVariableFields, NewCdna},
    },
    id::NoId,
    nucleic_acid_measurement::NewNucleicAcidMeasurement,
};
use nonempty::NonemptyVec;
use uuid::Uuid;

pub fn split_new_cdna_for_insertion(
    NewCdna {
        simple:
            CdnaSimpleFields {
                readable_id,
                prepared_at,
                additional_data,
            },
        gem_well_id,
        measurements,
        preparers,
        variable_fields,
    }: NewCdna,
) -> (
    NewCdnaRecord,
    Vec<NewNucleicAcidMeasurement>,
    NonemptyVec<Uuid>,
) {
    let library_type = variable_fields.into();

    let n_amplification_cycles = match variable_fields {
        CdnaVariableFields::GeneExpression {
            n_amplification_cycles,
        } => Some(n_amplification_cycles),
        CdnaVariableFields::AntibodyCapture
        | CdnaVariableFields::AntigenCapture
        | CdnaVariableFields::ChromatinAccessibility
        | CdnaVariableFields::CrisprGuideCapture
        | CdnaVariableFields::Custom
        | CdnaVariableFields::MultiplexingCapture
        | CdnaVariableFields::Vdj
        | CdnaVariableFields::VdjB
        | CdnaVariableFields::VdjT
        | CdnaVariableFields::VdjTGd => None,
    };

    (
        NewCdnaRecord {
            id: NoId,
            readable_id,
            library_type,
            prepared_at,
            gem_well_id: Some(gem_well_id),
            n_amplification_cycles,
            additional_data,
        },
        measurements,
        preparers,
    )
}
