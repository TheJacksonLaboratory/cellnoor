pub use common::{
    ControlledRateFreezing, DithiobisSuccinimidylpropionate, FlashFreezing, FormaldehydeDerivative,
};
use jiff::Timestamp;
use macro_attributes::{base_model, discriminant_unit_enum};
use uuid::Uuid;

use crate::{
    nonempty::NonemptyString,
    specimen::{
        Species,
        creation::{
            block::BlockFields, suspension::SuspensionSpecimenFields, tissue::TissueFields,
        },
        measurement::NewSpecimenMeasurement,
    },
};

pub mod block;
mod common;
pub mod suspension;
pub mod tissue;

#[base_model]
pub struct SpecimenSimpleFields {
    pub readable_id: NonemptyString,
    pub name: NonemptyString,
    pub received_or_created_at: Timestamp,
    pub transfer: Option<SpecimenTransfer>,
    pub additional_data: Option<serde_json::Value>,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub origin: SpecimenOrigin,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub variable_fields: SpecimenVariableFields,
}

#[base_model]
#[derive(Copy)]
pub struct SpecimenTransfer {
    pub given_at: Timestamp,
    pub given_by: Uuid,
    pub given_to: Uuid,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "origin"))]
pub enum SpecimenOrigin {
    Primary {
        submitted_by: Uuid,
        project_id: Uuid,
        species: Species,
        host_species: Option<Species>,
        tissue: NonemptyString,
    },
    Derived {
        parent_specimen_id: Uuid,
    },
}

#[base_model]
pub struct NewSpecimen {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: SpecimenSimpleFields,
    pub measurements: Vec<NewSpecimenMeasurement>,
}

pub type SpecimenUpdate = SpecimenSimpleFields;

#[base_model]
#[derive(Copy, strum::EnumDiscriminants)]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
#[strum(serialize_all = "snake_case")]
#[strum_discriminants(name(SpecimenType), discriminant_unit_enum)]
pub enum SpecimenVariableFields {
    Block(BlockFields),
    CellPellet {
        thermal_preservation_method: FlashFreezing,
    },
    RnaExtract,
    Suspension(SuspensionSpecimenFields),
    Tissue(TissueFields),
}
