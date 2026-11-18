use macro_attributes::{select, sql_enum};
pub use query::{SampleMultiplexingOperator, TenxAssayField, TenxAssayPredicate};
use uuid::Uuid;

use crate::{Relation, cdna::creation::LibraryType, nonempty::NonemptyString};

pub mod creation;
mod query;

#[sql_enum]
#[cfg_attr(feature = "postgres-types", postgres(name = "sample_multiplexing"))]
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
    pub sample_multiplexing: Option<SampleMultiplexing>,
    pub chemistry_version: NonemptyString,
    pub protocol_url: NonemptyString,
    pub chromium_chip: Option<NonemptyString>,
}

impl Relation for TenxAssay {
    const NAME: &'static str = "tenx_assay";
}
