use macro_attributes::{select, unit_enum};
use nonempty::NonemptyString;
pub use query::{SampleMultiplexingOperator, TenxAssayField, TenxAssayPredicate};
use uuid::Uuid;

use crate::cdna::creation::LibraryType;

pub mod creation;
mod query;

#[unit_enum]
pub enum SampleMultiplexing {
    Cellplex,
    FlexBarcode,
    FlexOligonucleotideBarcode,
    Hashtag,
    OnChipMultiplexing,
    Singleplex,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "tenx_assay"))]
pub struct TenxAssay {
    pub id: Uuid,
    pub name: NonemptyString,
    pub library_types: Option<Vec<LibraryType>>,
    pub sample_multiplexing: Option<SampleMultiplexing>,
    pub chemistry_version: NonemptyString,
    pub protocol_url: NonemptyString,
    pub chromium_chip: Option<NonemptyString>,
    pub cmdlines: Option<Vec<NonemptyString>>,
}
