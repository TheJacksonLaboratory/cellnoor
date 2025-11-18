use jiff::Timestamp;
use macro_attributes::{insert_select, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};
use non_empty_string::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::specimens;
use serde_json::Value;
use uuid::Uuid;

use crate::utils::{EnumFromSql, EnumToSql};

#[simple_enum]
enum Species {
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

impl EnumFromSql for Species {}

impl EnumToSql for Species {}

impl_enum_from_sql!(Species);

impl_enum_to_sql!(Species);

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = specimens))]
struct Fields {
    readable_id: NonEmptyString,
    name: NonEmptyString,
    submitted_by: Uuid,
    lab_id: Uuid,
    #[diesel(
        serialize_as = jiff_diesel::Timestamp,
        deserialize_as = jiff_diesel::Timestamp
    )]
    received_at: Timestamp,
    species: Species,
    host_species: Option<Species>,
    tissue: NonEmptyString,
    #[diesel(
        serialize_as = jiff_diesel::NullableTimestamp,
        deserialize_as = jiff_diesel::NullableTimestamp
    )]
    returned_at: Option<Timestamp>,
    returned_by: Option<Uuid>,
    additional_data: Option<Value>,
}
