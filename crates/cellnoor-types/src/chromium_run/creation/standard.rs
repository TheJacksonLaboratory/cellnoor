use macro_attributes::base_model;
use nonempty::NonemptyString;

use crate::chromium_run::creation::LoadedEntity;

#[base_model]
pub struct NewStandardGemWell {
    pub readable_id: NonemptyString,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub loaded_entity: LoadedEntity,
}
