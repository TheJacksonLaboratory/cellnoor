use macro_attributes::base_model;
use nonempty::NonemptyString;
use positive::PositiveU32;

use crate::tenx_assay::{LibraryType, creation::chromium::NewChromiumAssay};

mod chromium;

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "platform", rename_all = "snake_case"))]
pub enum NewTenxAssay {
    Chromium(NewChromiumAssay),
}

#[base_model]
pub struct LibraryTypeSpecification {
    pub library_type: LibraryType,
    pub index_kit: NonemptyString,
    pub cdna_volume_µl: PositiveU32,
    pub library_volume_µl: PositiveU32,
}
