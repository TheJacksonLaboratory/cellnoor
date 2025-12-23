#[cfg(feature = "app")]
use cellnoor_schema::tenx_assays;
#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::insert;
use non_empty::{NonEmptyString, NonEmptyVec};

use crate::tenx_assay::common::{
    LibraryType, LibraryTypeSpecification, SampleMultiplexing, TenxAssayFields,
};

#[insert]
#[cfg_attr(feature = "app", derive(AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = tenx_assays))]
pub struct ChromiumAssayCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: TenxAssayFields,
    sample_multiplexing: SampleMultiplexing,
    chromium_chip: NonEmptyString,
    #[cfg_attr(feature = "app", diesel(serialize_as = Vec<NonEmptyString>))]
    cmdlines: NonEmptyVec<NonEmptyString, { usize::MAX }>,
    #[cfg_attr(feature = "app", diesel(skip_insertion, skip_update))]
    library_type_specifications: NonEmptyVec<LibraryTypeSpecification, { usize::MAX }>,
}

impl ChromiumAssayCreation {
    pub fn protocol_url(&self) -> &str {
        self.inner.protocol_url()
    }

    pub fn library_type_specifications(&self) -> &[LibraryTypeSpecification] {
        self.library_type_specifications.as_ref()
    }

    pub fn library_types(&self) -> Vec<LibraryType> {
        self.library_type_specifications()
            .iter()
            .map(super::super::common::LibraryTypeSpecification::library_type)
            .collect()
    }
}
