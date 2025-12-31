#[cfg(feature = "app")]
use cellnoor_schema::tenx_assays;
use macro_attributes::select;
use uuid::Uuid;

use crate::{
    links::Links,
    tenx_assay::common::{LibraryType, SampleMultiplexing},
};

#[select]
pub struct TenxAssay {
    id: Uuid,
    links: Links,
    name: String,
    library_types: Option<Vec<Option<LibraryType>>>,
    sample_multiplexing: Option<SampleMultiplexing>,
    chemistry_version: String,
    protocol_url: String,
    chromium_chip: Option<String>,
    cmdlines: Option<Vec<Option<String>>>,
}

impl TenxAssay {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }
}
