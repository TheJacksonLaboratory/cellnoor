#[cfg(feature = "app")]
use cellnoor_schema::tenx_assays;
use macro_attributes::select;
use uuid::Uuid;

use crate::tenx_assay::common::{LibraryType, SampleMultiplexing};

#[select]
pub struct TenxAssay {
    id: Uuid,
    name: String,
    library_types: Option<Vec<Option<LibraryType>>>,
    sample_multiplexing: Option<SampleMultiplexing>,
    chemistry_version: String,
    protocol_url: String,
    chromium_chip: Option<String>,
    cmdlines: Option<Vec<Option<String>>>,
    #[cfg_attr(feature = "app", diesel(embed))]
    links: TenxAssayLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = tenx_assays))]
pub struct TenxAssayLinks {
    #[serde(rename = "self")]
    self_link: String,
}

impl TenxAssay {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }
}
