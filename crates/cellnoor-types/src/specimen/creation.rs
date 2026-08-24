use jiff::Timestamp;
use macro_attributes::{base_model, discriminant_unit_enum};
use nonempty::NonemptyString;
use uuid::Uuid;

use crate::specimen::{
        Species,
        creation::{
            block::BlockFields,
            suspension::SuspensionSpecimenFields,
            tissue::TissueFields,
        },
        measurement::NewSpecimenMeasurement,
    };

pub use common::{
    ControlledRateFreezing, DithiobisSuccinimidylpropionate, FlashFreezing, FormaldehydeDerivative,
};

pub mod block;
mod common;
pub mod suspension;
pub mod tissue;

#[base_model]
pub struct NewSpecimen {
    pub readable_id: NonemptyString,
    pub name: NonemptyString,
    pub submitted_by: Uuid,
    pub project_id: Uuid,
    pub received_at: Timestamp,
    pub species: Species,
    pub host_species: Option<Species>,
    pub returned_at: Option<Timestamp>,
    pub returned_by: Option<Uuid>,
    pub tissue: NonemptyString,
    pub additional_data: Option<serde_json::Value>,
    pub measurements: Vec<NewSpecimenMeasurement>,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub variable_fields: SpecimenVariableFields,
}

#[base_model]
#[derive(Copy, strum::EnumDiscriminants)]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
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
