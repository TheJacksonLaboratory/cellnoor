#[cfg(feature = "postgres-types")]
use std::str::FromStr;

#[cfg(feature = "postgres-types")]
use bytes::BytesMut;
pub use creation::SpecimenType;
use macro_attributes::{base_model, select, sql_enum};
#[cfg(feature = "postgres-types")]
use postgres_types::{FromSql, ToSql, to_sql_checked};
pub use query::{
    BlockEmbeddingMatrixOperator, FixativeOperator, SimpleSpecimenQuery, SpeciesOperator,
    SpecimenField, SpecimenPredicate, SpecimenQuery, SpecimenTypeOperator,
    ThermalPreservationMethodOperator,
};

use crate::{
    Relation,
    id::Id,
    project::{ProjectCompact, SavedProjectRecord},
    simple_links::SimpleLinks,
    specimen::{
        creation::{
            ControlledRateFreezing, DithiobisSuccinimidylpropionate, FlashFreezing,
            FormaldehydeDerivative,
        },
        measurement::SpecimenMeasurement,
        record::SpecimenRecord,
    },
};

pub mod creation;
pub mod measurement;
mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    use uuid::Uuid;

    use crate::{
        nonempty::NonemptyString,
        project::ProjectActivePeriod,
        specimen::{
            Fixative, SpecimenSpecies, SpecimenType, ThermalPreservationMethod,
            creation::block::BlockEmbeddingMatrix,
        },
    };

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "specimen"))]
    pub struct SpecimenRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub readable_id: NonemptyString,
        pub name: NonemptyString,
        pub submitted_by: Option<Uuid>,
        pub project_id: Uuid,
        #[cfg_attr(feature = "serde", serde(skip))]
        pub project_active_period: ProjectActivePeriod,
        pub received_or_created_at: Timestamp,
        pub parent_specimen_id: Option<Uuid>,
        #[cfg_attr(feature = "serde", serde(skip))]
        pub parent_specimen_received_or_created_at: Option<Timestamp>,
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub species: SpecimenSpecies,
        pub given_at: Option<Timestamp>,
        pub given_by: Option<Uuid>,
        pub given_to: Option<Uuid>,
        #[cfg_attr(feature = "postgres-types", postgres(name = "type"))]
        pub type_: SpecimenType,
        pub embedded_in: Option<BlockEmbeddingMatrix>,
        pub fixative: Option<Fixative>,
        pub thermal_preservation_method: Option<ThermalPreservationMethod>,
        pub tissue: NonemptyString,
        pub additional_data: Option<serde_json::Value>,
    }
}

#[select]
#[derive(Copy)]
#[cfg_attr(feature = "postgres-types", postgres(name = "specimen_species"))]
pub struct SpecimenSpecies {
    pub species: Species,
    pub host_species: Option<Species>,
}

#[sql_enum]
#[cfg_attr(feature = "postgres-types", postgres(name = "species"))]
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

impl<T> Relation for SpecimenRecord<T> {
    const NAME: &'static str = "specimen";
}

pub type SavedSpecimenRecord = SpecimenRecord<Id>;

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "specimen_detailed"))]
pub struct SavedSpecimenRecordDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub specimen: SavedSpecimenRecord,
    pub project: SavedProjectRecord,
    pub measurements: Vec<SpecimenMeasurement>,
    pub ancestors: Vec<SavedSpecimenRecord>,
}

#[base_model]
pub struct SpecimenCompact {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedSpecimenRecord,
    pub links: SimpleLinks,
}

// Rather than just wrapping `SavedSpecimenRecordDetailed`, we destructure its
// fields so that we have a `ProjectCompact` rather than a bare
// `SavedProjectRecord`.
#[base_model]
pub struct SpecimenDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedSpecimenRecord,
    pub links: SimpleLinks,
    pub project: ProjectCompact,
    pub measurements: Vec<SpecimenMeasurement>,
    pub ancestors: Vec<SpecimenCompact>,
}

#[sql_enum]
#[cfg_attr(feature = "postgres-types", postgres(name = "thermal_preservation_method"))]
pub enum ThermalPreservationMethod {
    ControlledRateFreezing,
    FlashFreezing,
}

#[sql_enum]
#[cfg_attr(feature = "postgres-types", postgres(name = "fixative"))]
pub enum Fixative {
    DithiobisSuccinimidylpropionate,
    FormaldehydeDerivative,
}
