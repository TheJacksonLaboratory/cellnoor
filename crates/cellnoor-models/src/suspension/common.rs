#[cfg(feature = "app")]
use cellnoor_schema::suspensions;
use macro_attributes::{insert_select, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};
use non_empty::NonEmptyString;
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
pub struct SuspensionFields {
    pub(super) readable_id: NonEmptyString,
    pub(super) parent_specimen_id: Uuid,
    pub(super) additional_data: Option<Value>,
}
