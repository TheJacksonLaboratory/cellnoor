use macro_attributes::insert;
use non_empty::{NonEmptyString, NonEmptyVec};
#[cfg(feature = "app")]
use scamplers_schema::tenx_assays;

use crate::tenx_assay::common::{LibraryTypeSpecification, SampleMultiplexing, TenxAssayFields};

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = tenx_assays))]
pub struct ChromiumAssayCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: TenxAssayFields,
    sample_multiplexing: SampleMultiplexing,
    chromium_chip: NonEmptyString,
    #[cfg_attr(feature = "app", diesel(serialize_as = Vec::<NonEmptyString>))]
    cmdlines: NonEmptyVec<NonEmptyString>,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    library_type_specifications: NonEmptyVec<LibraryTypeSpecification>,
}

impl ChromiumAssayCreation {
    pub fn protocol_url(&self) -> &str {
        self.inner.protocol_url()
    }
}
