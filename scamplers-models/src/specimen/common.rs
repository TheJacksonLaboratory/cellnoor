use macro_attributes::{insert_select, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};
use non_empty_string::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::specimens;
use serde_json::Value;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

#[simple_enum]
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

#[cfg(feature = "app")]
impl EnumToSql for Species {}

impl_enum_from_sql!(Species);

impl_enum_to_sql!(Species);

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = specimens))]
pub struct Fields {
    readable_id: NonEmptyString,
    name: NonEmptyString,
    submitted_by: Uuid,
    lab_id: Uuid,
    species: Species,
    host_species: Option<Species>,
    returned_by: Option<Uuid>,
    tissue: NonEmptyString,
    additional_data: Option<Value>,
}

pub(super) const fn true_() -> bool {
    true
}
