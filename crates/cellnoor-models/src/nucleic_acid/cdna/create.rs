#[cfg(feature = "app")]
use cellnoor_schema::cdna;
use jiff::Timestamp;
use macro_attributes::insert;
use non_empty::NonEmptyVec;
use uuid::Uuid;

use crate::{nucleic_acid::cdna::common::CdnaFields, tenx_assay::LibraryType};

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = cdna))]
pub struct CdnaCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: CdnaFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    prepared_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(serialize_as = i32))]
    n_amplification_cycles: u8,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    volume_µl: u8,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    preparer_ids: NonEmptyVec<Uuid, { usize::MAX }>,
}

impl CdnaCreation {
    #[must_use]
    pub fn gem_pool_id(&self) -> Option<Uuid> {
        self.inner.gem_pool_id
    }

    #[must_use]
    pub fn library_type(&self) -> LibraryType {
        self.inner.library_type
    }

    #[must_use]
    pub fn volume_µl(&self) -> u8 {
        self.volume_µl
    }

    #[must_use]
    pub fn prepared_at(&self) -> Timestamp {
        self.prepared_at
    }

    #[must_use]
    pub fn preparer_ids(&self) -> &[Uuid] {
        self.preparer_ids.as_ref()
    }
}
