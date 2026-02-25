#[cfg(feature = "app")]
use cellnoor_schema::specimens;
use macro_attributes::{insert_select, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};

use super::creation::block::BlockEmbeddingMatrix;
#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

#[simple_enum]
pub enum SpecimenType {
    Block,
    CellPellet,
    Suspension,
    Tissue,
}

#[cfg(feature = "app")]
impl EnumFromSql for SpecimenType {}
impl_enum_from_sql!(SpecimenType);

#[cfg(feature = "app")]
impl EnumToSql for SpecimenType {}
impl_enum_to_sql!(SpecimenType);

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum Fixative {
    DithiobisSuccinimidylpropionate,
    FormaldehydeDerivative,
}

#[cfg(feature = "app")]
impl EnumFromSql for Fixative {}
impl_enum_from_sql!(Fixative);

#[cfg(feature = "app")]
impl EnumToSql for Fixative {}
impl_enum_to_sql!(Fixative);

#[simple_enum]
#[derive(strum::VariantArray)]
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
