use jiff::Timestamp;
use macro_attributes::{insert_select, json};
use macros::{impl_json_from_sql, impl_json_to_sql};
use non_empty::NonEmptyString;
use positive::PositiveF32;
#[cfg(feature = "app")]
use scamplers_schema::{chip_loadings, chromium_runs, gem_pools};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    units::Microliter,
    utils::{JsonFromSql, JsonToSql},
};

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = gem_pools))]
pub struct GemPoolFields {
    readable_id: NonEmptyString,
}

#[json]
pub struct Volume {
    value: PositiveF32,
    unit: Microliter,
}

#[cfg(feature = "app")]
impl JsonFromSql for Volume {}
impl_json_from_sql!(Volume);

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
    readable_id: NonEmptyString,
    assay_id: Uuid,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp, deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    run_at: Timestamp,
    run_by: Uuid,
    succeeded: bool,
    additional_data: Option<Value>,
}
