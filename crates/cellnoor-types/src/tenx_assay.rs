use macro_attributes::{predicate_enum, select, sort_field_enum, unit_enum};
use nonempty::NonemptyString;
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;
use uuid::Uuid;

use crate::operator::{StringOperator, UuidOperator};
#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;

#[unit_enum]
pub enum LibraryType {
    AntibodyCapture,
    AntigenCapture,
    ChromatinAccessibility,
    CrisprGuideCapture,
    Custom,
    GeneExpression,
    MultiplexingCapture,
    Vdj,
    VdjB,
    VdjT,
    VdjTGd,
}

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

#[predicate_enum]
#[strum(prefix = "(tenx_assay).")]
#[strum_discriminants(name(TenxAssayField), sort_field_enum, strum(prefix = "(tenx_assay)."))]
pub enum TenxAssayPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    ChemistryVersion(StringOperator),
    ProtocolUrl(StringOperator),
}

#[cfg(feature = "postgres-types")]
impl ToPredicate for TenxAssayPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Id(u) => u.to_predicate(),
            Self::Name(s) | Self::ChemistryVersion(s) | Self::ProtocolUrl(s) => s.to_predicate(),
        }
    }
}

impl Default for TenxAssayField {
    fn default() -> Self {
        Self::Name
    }
}
