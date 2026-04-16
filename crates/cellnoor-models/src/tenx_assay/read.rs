#[cfg(feature = "app")]
use cellnoor_schema::tenx_assays;
use macro_attributes::select;
use uuid::Uuid;

use crate::tenx_assay::common::{LibraryType, SampleMultiplexing};

#[select]
pub struct TenxAssay {
    pub id: Uuid,
    pub name: String,
    pub library_types: Option<Vec<Option<LibraryType>>>,
    pub sample_multiplexing: Option<SampleMultiplexing>,
    pub chemistry_version: String,
    pub protocol_url: String,
    pub chromium_chip: Option<String>,
    pub cmdlines: Option<Vec<Option<String>>>,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub links: TenxAssayLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = tenx_assays))]
pub struct TenxAssayLinks {
    #[serde(rename = "self")]
    pub self_link: String,
}

impl TenxAssay {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }
}
