use std::str::FromStr;

#[cfg(feature = "app")]
use cellnoor_schema::specimens;
use macro_attributes::{base_model, insert_select, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};

use super::creation::{
    block::{BlockEmbeddingMatrix, BlockFixative},
    suspension::SuspensionFixative,
    tissue::TissueFixative,
};
#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

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
pub enum ThermalPreservationMethod {
    ControlledRateFreezing,
    FlashFreezing,
}

#[cfg(feature = "app")]
impl EnumFromSql for ThermalPreservationMethod {}
impl_enum_from_sql!(ThermalPreservationMethod);

#[cfg(feature = "app")]
impl EnumToSql for ThermalPreservationMethod {}
impl_enum_to_sql!(ThermalPreservationMethod);

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = specimens))]
pub struct SpecimenVariableFields {
    pub(crate) type_: SpecimenType,
    pub(crate) embedded_in: Option<BlockEmbeddingMatrix>,
    pub(crate) fixative: Option<Fixative>,
    pub(crate) thermal_preservation_method: Option<ThermalPreservationMethod>,
}
