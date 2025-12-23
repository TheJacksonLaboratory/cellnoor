#[cfg(feature = "app")]
use cellnoor_schema::suspension_pools;
use jiff::Timestamp;
use macro_attributes::insert_select;
use non_empty::NonEmptyString;
use serde_json::Value;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = suspension_pools))]
pub struct SuspensionPoolFields {
    readable_id: NonEmptyString,
    name: NonEmptyString,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp, deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    pooled_at: Timestamp,
    additional_data: Option<Value>,
}

impl SuspensionPoolFields {
    #[must_use]
    pub fn pooled_at(&self) -> Timestamp {
        self.pooled_at
    }
}
