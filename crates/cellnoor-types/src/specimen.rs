pub use creation::{
    NewSpecimen,
    block::{BlockEmbeddingMatrix, BlockFixative, NewBlock},
    suspension::{NewSuspensionSpecimen, SuspensionThermalPreservation},
    tissue::NewTissue,
};
use jiff::Timestamp;
use macro_attributes::{base_model, select, unit_enum};
use nonempty::NonemptyString;
pub use query::SpecimenQuery;
pub use read::SpecimenRecord;
use serde_json::Value;
use uuid::Uuid;

mod creation;
pub mod measurement;
mod query;
mod read;

#[unit_enum]
pub enum Species {
    AmbystomaMexicanum,
    CanisFamiliaris,
    CallithrixJacchus,
    DrosophilaMelanogaster,
    GasterosteusAculeatus,
    HomoSapiens,
    MusMusculus,
    RattusNorvegicus,
    SminthopsisCrassicaudata,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "specimen_common_fields"))]
pub struct SpecimenCommonFields {
    pub readable_id: NonemptyString,
    pub name: NonemptyString,
    pub submitted_by: Uuid,
    pub received_at: Timestamp,
    pub project_id: Uuid,
    pub species: Species,
    pub host_species: Option<Species>,
    pub returned_by: Option<Uuid>,
    pub returned_at: Option<Timestamp>,
    pub tissue: NonemptyString,
    pub additional_data: Option<Value>,
}

#[unit_enum]
pub enum SpecimenType {
    Block,
    CellPellet,
    RnaExtract,
    Suspension,
    Tissue,
}

#[unit_enum]
pub enum Fixative {
    DithiobisSuccinimidylpropionate,
    FormaldehydeDerivative,
}

#[unit_enum]
pub enum ThermalPreservationMethod {
    ControlledRateFreezing,
    FlashFreezing,
}

#[select]
#[cfg_attr(
    feature = "postgres-types",
    postgres(name = "specimen_variable_fields")
)]
pub struct SpecimenVariableFields {
    pub type_: SpecimenType,
    pub embedded_in: Option<BlockEmbeddingMatrix>,
    pub fixative: Option<Fixative>,
    pub thermal_preservation_method: Option<ThermalPreservationMethod>,
}

#[base_model]
pub struct Specimen {
    pub id: Uuid,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub common: SpecimenCommonFields,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub variable: SpecimenVariableFields,
}
