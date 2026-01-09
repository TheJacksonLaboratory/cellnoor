#[cfg(feature = "app")]
use cellnoor_schema::specimens;
use jiff::Timestamp;
use macro_attributes::{insert_select, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};
use non_empty::NonEmptyString;
use serde_json::Value;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

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
