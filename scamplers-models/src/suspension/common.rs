use jiff::Timestamp;
use macro_attributes::{insert_select, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};
use non_empty::NonEmptyString;
use positive::{PositiveF32, PositiveU32};
#[cfg(feature = "app")]
use scamplers_schema::suspensions;
use serde_json::Value;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum SuspensionContent {
    Cells,
    Nuclei,
}

#[cfg(feature = "app")]
impl EnumFromSql for SuspensionContent {}
impl_enum_from_sql!(SuspensionContent);

#[cfg(feature = "app")]
impl EnumToSql for SuspensionContent {}
impl_enum_to_sql!(SuspensionContent);

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = suspensions))]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct SuspensionFields {
    readable_id: NonEmptyString,
    parent_specimen_id: Uuid,
    target_cell_recovery: PositiveU32,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::NullableTimestamp,
        deserialize_as = jiff_diesel::NullableTimestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    created_at: Option<Timestamp>,
    lysis_duration_minutes: Option<PositiveF32>,
    additional_data: Option<Value>,
}
