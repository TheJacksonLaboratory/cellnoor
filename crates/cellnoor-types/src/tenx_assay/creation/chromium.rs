use macro_attributes::base_model;
use nonempty::{NonemptyString, NonemptyVec};

use crate::tenx_assay::{SampleMultiplexing, creation::LibraryTypeSpecification};

#[base_model]
pub struct NewChromiumAssay {
    pub name: NonemptyString,
    pub chemistry_version: NonemptyString,
    pub protocol_url: NonemptyString,
    pub sample_multiplexing: SampleMultiplexing,
    pub chromium_chip: NonemptyString,
    pub cmdlines: NonemptyVec<NonemptyString>,
    pub library_type_specifications: NonemptyVec<LibraryTypeSpecification>,
}
