use cellnoor_types::{
    id::NoId,
    specimen::{
        Fixative, NewSpecimenRecord, ThermalPreservationMethod,
        creation::{
            SpecimenSimpleFields, SpecimenUpdate, SpecimenVariableFields, block::BlockFields,
            suspension::SuspensionSpecimenFields, tissue::TissueFields,
        },
        measurement::NewSpecimenMeasurement,
    },
};

pub fn convert_specimen_fields_to_record(
    SpecimenSimpleFields {
        readable_id,
        name,
        submitted_by,
        project_id,
        received_at,
        species,
        host_species,
        returned_at,
        returned_by,
        tissue,
        additional_data,
        variable_fields,
    }: SpecimenSimpleFields,
) -> NewSpecimenRecord {
    let type_ = variable_fields.into();

    let embedded_in = match variable_fields {
        SpecimenVariableFields::Block(b) => Some(b.into()),
        SpecimenVariableFields::CellPellet { .. }
        | SpecimenVariableFields::RnaExtract
        | SpecimenVariableFields::Tissue(..)
        | SpecimenVariableFields::Suspension(..) => None,
    };

    let (fixative, thermal_preservation_method) = match variable_fields {
        SpecimenVariableFields::Block(
            BlockFields::CarboxymethylCellulose {
                fixative,
                thermal_preservation_method,
            }
            | BlockFields::OptimalCuttingTemperatureCompound {
                fixative,
                thermal_preservation_method,
            },
        ) => (
            fixative.map(Fixative::FormaldehydeDerivative),
            Some(ThermalPreservationMethod::Ff(thermal_preservation_method)),
        ),
        SpecimenVariableFields::Block(BlockFields::Paraffin { fixative }) => {
            (Some(Fixative::FormaldehydeDerivative(fixative)), None)
        }
        SpecimenVariableFields::CellPellet {
            thermal_preservation_method,
        } => (
            None,
            Some(ThermalPreservationMethod::Ff(thermal_preservation_method)),
        ),
        SpecimenVariableFields::RnaExtract
        | SpecimenVariableFields::Suspension(SuspensionSpecimenFields::Fresh)
        | SpecimenVariableFields::Tissue(TissueFields::Fresh) => (None, None),
        SpecimenVariableFields::Suspension(SuspensionSpecimenFields::Fixed { fixative })
        | SpecimenVariableFields::Tissue(TissueFields::Fixed { fixative }) => {
            (Some(fixative), None)
        }
        SpecimenVariableFields::Suspension(SuspensionSpecimenFields::ThermallyPreserved {
            thermal_preservation_method,
        }) => (
            None,
            Some(ThermalPreservationMethod::Crf(thermal_preservation_method)),
        ),
        SpecimenVariableFields::Tissue(TissueFields::ThermallyPreserved {
            thermal_preservation_method,
        }) => (None, Some(thermal_preservation_method)),
    };

    NewSpecimenRecord {
        id: NoId,
        readable_id,
        name,
        submitted_by,
        project_id,
        // These are denormalized copies of the project's start and end,
        // populated by a database trigger on insert. The values here are never
        // written or read
        project_started_at: Default::default(),
        project_ended_at: Default::default(),
        received_at,
        species,
        host_species,
        returned_at,
        returned_by,
        tissue,
        additional_data,
        embedded_in,
        fixative,
        thermal_preservation_method,
        type_,
    }
}
