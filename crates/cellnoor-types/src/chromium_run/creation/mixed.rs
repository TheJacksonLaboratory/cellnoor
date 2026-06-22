use macro_attributes::base_model;

use crate::chromium_run::creation::{ocm::NewOcmGemWell, standard::NewStandardGemWell};

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "plexy", rename_all = "snake_case"))]
pub enum NewStandardOrOcmGemWell {
    OnChipMultiplexing(NewOcmGemWell),
    Standard(NewStandardGemWell),
}
