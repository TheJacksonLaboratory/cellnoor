use macro_attributes::base_model;

use crate::tenx_assay::creation::chromium::ChromiumAssayCreation;

mod chromium;

#[base_model]
#[serde(tag = "platform", rename_all = "snake_case")]
pub enum TenxAssayCreation {
    Chromium(ChromiumAssayCreation),
}

impl TenxAssayCreation {
    #[must_use]
    pub fn protocol_url(&self) -> &str {
        match self {
            Self::Chromium(a) => a.protocol_url(),
        }
    }
}
