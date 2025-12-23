use std::str::FromStr;

#[cfg(feature = "app")]
use cellnoor_schema::specimens;
use jiff::Timestamp;
use macro_attributes::{base_model, insert_select, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};
use non_empty::NonEmptyString;
use serde_json::Value;
use uuid::Uuid;

use crate::specimen::creation::{
    block::{BlockFixative, FixedBlockEmbeddingMatrix, FrozenBlockEmbeddingMatrix},
    suspension::SuspensionFixative,
    tissue::TissueFixative,
};
#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = specimens))]
pub struct SpecimenCommonFields {
    pub(super) readable_id: NonEmptyString,
    pub(super) name: NonEmptyString,
    pub(super) submitted_by: Uuid,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::Timestamp,
        deserialize_as = jiff_diesel::Timestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    pub(super) received_at: Timestamp,
    pub(super) lab_id: Uuid,
    pub(super) species: Species,
    pub(super) host_species: Option<Species>,
    pub(super) returned_by: Option<Uuid>,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::NullableTimestamp,
        deserialize_as = jiff_diesel::NullableTimestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    pub(super) returned_at: Option<Timestamp>,
    pub(super) tissue: NonEmptyString,
    pub(super) additional_data: Option<Value>,
}

#[simple_enum]
pub enum SpecimenType {
    Block,
    Suspension,
    Tissue,
}

#[cfg(feature = "app")]
impl EnumFromSql for SpecimenType {}
impl_enum_from_sql!(SpecimenType);

#[cfg(feature = "app")]
impl EnumToSql for SpecimenType {}
impl_enum_to_sql!(SpecimenType);

#[base_model]
#[derive(Copy, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
#[cfg_attr(
    feature = "app",
    derive(::diesel::deserialize::FromSqlRow, ::diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "app", diesel(sql_type = ::diesel::sql_types::Text))]
#[serde(untagged)]
pub enum EmbeddingMatrix {
    FixedBlock(FixedBlockEmbeddingMatrix),
    FrozenBlock(FrozenBlockEmbeddingMatrix),
}

impl FromStr for EmbeddingMatrix {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FixedBlockEmbeddingMatrix::from_str(s)
            .map(Self::FixedBlock)
            .or_else(|_| FrozenBlockEmbeddingMatrix::from_str(s).map(Self::FrozenBlock))
    }
}

#[cfg(feature = "app")]
impl EnumFromSql for EmbeddingMatrix {}
impl_enum_from_sql!(EmbeddingMatrix);

impl From<&EmbeddingMatrix> for &'static str {
    fn from(embedding_matrix: &EmbeddingMatrix) -> &'static str {
        use EmbeddingMatrix::{FixedBlock, FrozenBlock};

        match embedding_matrix {
            FixedBlock(em) => em.into(),
            FrozenBlock(em) => em.into(),
        }
    }
}

#[cfg(feature = "app")]
impl EnumToSql for EmbeddingMatrix {}
impl_enum_to_sql!(EmbeddingMatrix);

#[base_model]
#[derive(Copy, serde::Deserialize, serde::Serialize)]
#[cfg_attr(
    feature = "app",
    derive(::diesel::deserialize::FromSqlRow, ::diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "app", diesel(sql_type = ::diesel::sql_types::Text))]
#[serde(untagged)]
pub enum Fixative {
    Block(BlockFixative),
    Suspension(SuspensionFixative),
    Tissue(TissueFixative),
}

impl FromStr for Fixative {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BlockFixative::from_str(s)
            .map(Self::Block)
            .or_else(|_| SuspensionFixative::from_str(s).map(Self::Suspension))
            .or_else(|_| TissueFixative::from_str(s).map(Self::Tissue))
    }
}

#[cfg(feature = "app")]
impl EnumFromSql for Fixative {}
impl_enum_from_sql!(Fixative);

impl From<&Fixative> for &'static str {
    fn from(fixative: &Fixative) -> &'static str {
        use Fixative::{Block, Suspension, Tissue};

        match fixative {
            Block(f) => f.into(),
            Suspension(f) => f.into(),
            Tissue(f) => f.into(),
        }
    }
}

#[cfg(feature = "app")]
impl EnumToSql for Fixative {}
impl_enum_to_sql!(Fixative);

#[simple_enum]
#[derive(strum::VariantArray)]
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

#[cfg(feature = "app")]
impl EnumFromSql for Species {}
impl_enum_from_sql!(Species);

#[cfg(feature = "app")]
impl EnumToSql for Species {}
impl_enum_to_sql!(Species);

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = specimens))]
pub struct SpecimenVariableFields {
    pub(crate) type_: SpecimenType,
    pub(crate) embedded_in: Option<EmbeddingMatrix>,
    pub(crate) fixative: Option<Fixative>,
    pub(crate) frozen: bool,
    pub(crate) cryopreserved: bool,
}
