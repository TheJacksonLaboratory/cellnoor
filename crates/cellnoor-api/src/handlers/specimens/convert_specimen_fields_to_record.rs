use cellnoor_types::specimen::{
    Fixative, ThermalPreservationMethod,
    creation::{
        SpecimenSimpleFields, SpecimenTransfer, SpecimenVariableFields, block::BlockFields,
        suspension::SuspensionSpecimenFields, tissue::TissueFields,
    },
};

use crate::handlers::specimens::create::NewSpecimenRecord;

pub(super) fn convert_specimen_fields_to_record(
    SpecimenSimpleFields {
        readable_id,
        name,
        received_or_created_at,
        transfer,
        additional_data,
        origin,
        variable_fields,
    }: SpecimenSimpleFields,
) -> NewSpecimenRecord {
    let type_ = variable_fields.into();

    let (given_at, given_by, given_to) = match transfer {
        Some(SpecimenTransfer {
            given_at,
            given_by,
            given_to,
        }) => (Some(given_at), Some(given_by), Some(given_to)),
        None => (None, None, None),
    };

    let embedded_in = match variable_fields {
        SpecimenVariableFields::Block(b) => Some(b.into()),
        SpecimenVariableFields::CellPellet { .. }
        | SpecimenVariableFields::RnaExtract
        | SpecimenVariableFields::Tissue(..)
        | SpecimenVariableFields::Suspension(..) => None,
    };

    let (fixative, thermal_preservation_method) = match variable_fields {
        SpecimenVariableFields::Block(
            BlockFields::CarboxymethylCellulose { fixative, .. }
            | BlockFields::OptimalCuttingTemperatureCompound { fixative, .. },
        ) => (
            fixative.map(|_| Fixative::FormaldehydeDerivative),
            Some(ThermalPreservationMethod::FlashFreezing),
        ),
        SpecimenVariableFields::Block(BlockFields::Paraffin { .. }) => {
            (Some(Fixative::FormaldehydeDerivative), None)
        }
        SpecimenVariableFields::CellPellet { .. } => {
            (None, Some(ThermalPreservationMethod::FlashFreezing))
        }
        SpecimenVariableFields::RnaExtract
        | SpecimenVariableFields::Suspension(SuspensionSpecimenFields::Fresh)
        | SpecimenVariableFields::Tissue(TissueFields::Fresh) => (None, None),
        SpecimenVariableFields::Suspension(SuspensionSpecimenFields::Fixed { fixative })
        | SpecimenVariableFields::Tissue(TissueFields::Fixed { fixative }) => {
            (Some(fixative), None)
        }
        SpecimenVariableFields::Suspension(SuspensionSpecimenFields::ThermallyPreserved {
            ..
        }) => (None, Some(ThermalPreservationMethod::ControlledRateFreezing)),
        SpecimenVariableFields::Tissue(TissueFields::ThermallyPreserved {
            thermal_preservation_method,
        }) => (None, Some(thermal_preservation_method)),
    };

    NewSpecimenRecord {
        readable_id,
        name,
        origin,
        received_or_created_at,
        given_at,
        given_by,
        given_to,
        type_,
        embedded_in,
        fixative,
        thermal_preservation_method,
        additional_data,
    }
}
