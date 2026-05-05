use macro_attributes::base_model;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[base_model]
#[derive(Copy, Hash, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct IdParam {
    pub id: Uuid,
}
