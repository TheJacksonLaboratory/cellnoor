use macro_attributes::base_model;
#[cfg(feature = "app")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[base_model]
#[derive(Serialize, Deserialize, Copy, Hash, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "app", derive(JsonSchema))]
#[cfg_attr(feature = "app", schemars(inline))]
pub struct IdParameter {
    pub id: Uuid,
}
