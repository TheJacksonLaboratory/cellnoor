#[cfg(feature = "app")]
use cellnoor_schema::{chip_loadings, chromium_runs, gem_pools};
use jiff::Timestamp;
use macro_attributes::{insert_select, json};
use macros::{impl_json_from_sql, impl_json_to_sql};
use non_empty::NonEmptyString;
use serde_json::Value;
use uuid::Uuid;

use crate::units::Microliter;
#[cfg(feature = "app")]
use crate::utils::{JsonFromSql, JsonToSql};

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = gem_pools))]
pub struct GemPoolFields {
    readable_id: NonEmptyString,
}

#[json]
pub struct Volume {
    value: u8,
    unit: Microliter,
}

impl Volume {
    #[must_use]
    pub fn new(value: u8) -> Self {
        Self {
            value,
            unit: Microliter::Microliter,
        }
    }
}

#[cfg(feature = "app")]
impl JsonFromSql for Volume {}
impl_json_from_sql!(Volume);

#[cfg(feature = "app")]
impl JsonToSql for Volume {}
impl_json_to_sql!(Volume);

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = chip_loadings))]
pub struct ChipLoadingFields {
    suspension_volume_loaded: Volume,
    buffer_volume_loaded: Volume,
    additional_data: Option<Value>,
}

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = chromium_runs))]
pub struct ChromiumRunFields {
    pub(super) readable_id: NonEmptyString,
    pub(super) assay_id: Uuid,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp, deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    pub(super) run_at: Timestamp,
    pub(super) run_by: Uuid,
    pub(super) succeeded: bool,
    pub(super) additional_data: Option<Value>,
}
