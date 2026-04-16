#[cfg(feature = "app")]
use cellnoor_schema::suspension_pools;
use jiff::Timestamp;
use macro_attributes::insert_select;
use non_empty::NonEmptyString;
use serde_json::Value;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = suspension_pools))]
pub struct SuspensionPoolFields {
    pub readable_id: NonEmptyString,
    pub name: NonEmptyString,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp, deserialize_as = jiff_diesel::Timestamp))]
    pub pooled_at: Timestamp,
    pub additional_data: Option<Value>,
}

impl SuspensionPoolFields {
    #[must_use]
    pub fn pooled_at(&self) -> Timestamp {
        self.pooled_at
    }
}
