use macro_attributes::base_model;

use crate::tenx_assay::{
    common::LibraryTypeSpecification, creation::chromium::ChromiumAssayCreation,
};

mod chromium;

#[base_model]
#[derive(serde::Deserialize)]
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

    #[must_use]
    pub fn library_type_specifications(&self) -> Option<&[LibraryTypeSpecification]> {
        match self {
            Self::Chromium(a) => Some(a.library_type_specifications()),
        }
    }
}
