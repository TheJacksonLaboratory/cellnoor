#[cfg(feature = "app")]
use cellnoor_schema::library_type_specifications;
#[cfg(feature = "app")]
use cellnoor_schema::tenx_assays;
#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::{insert, insert_select, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};
use non_empty::NonEmptyString;
use ranged::RangedU16;

#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

#[simple_enum]
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

#[cfg(feature = "app")]
impl EnumFromSql for LibraryType {}
impl_enum_from_sql!(LibraryType);

#[cfg(feature = "app")]
impl EnumToSql for LibraryType {}
impl_enum_to_sql!(LibraryType);

#[simple_enum]
pub enum SampleMultiplexing {
    Cellplex,
    FlexBarcode,
    FlexOligonucleotideBarcode,
    Hashtag,
    OnChipMultiplexing,
    Singleplex,
}

#[cfg(feature = "app")]
impl EnumFromSql for SampleMultiplexing {}
impl_enum_from_sql!(SampleMultiplexing);

#[cfg(feature = "app")]
impl EnumToSql for SampleMultiplexing {}
impl_enum_to_sql!(SampleMultiplexing);

#[insert]
#[cfg_attr(feature = "app", derive(AsChangeset, HasQuery))]
pub struct LibraryTypeSpecification {
    pub library_type: LibraryType,
    pub index_kit: String,
    #[cfg_attr(feature = "app", diesel(column_name = cdna_volume_l))]
    pub cdna_volume_µl: RangedU16<0, { u16::MAX }>,
    #[cfg_attr(feature = "app", diesel(column_name = library_volume_l))]
    pub library_volume_µl: RangedU16<0, { u16::MAX }>,
}

impl LibraryTypeSpecification {
    #[must_use]
    pub fn library_type(&self) -> LibraryType {
        self.library_type
    }

    #[must_use]
    pub fn index_kit(&self) -> &str {
        &self.index_kit
    }

    #[must_use]
    pub fn cdna_volume_µl(&self) -> u16 {
        self.cdna_volume_µl.into()
    }

    #[must_use]
    pub fn library_volume_µl(&self) -> u16 {
        self.library_volume_µl.into()
    }
}

#[insert_select]
#[cfg_attr(feature = "app", derive(AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = tenx_assays))]
pub struct TenxAssayFields {
    pub name: NonEmptyString,
    pub chemistry_version: NonEmptyString,
    pub protocol_url: NonEmptyString,
}

impl TenxAssayFields {
    pub fn protocol_url(&self) -> &str {
        self.protocol_url.as_ref()
    }
}
