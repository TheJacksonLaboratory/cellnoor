use macro_attributes::{select, unit_enum};
use nonempty::NonemptyString;
use uuid::Uuid;

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
