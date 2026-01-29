#[cfg(feature = "app")]
use cellnoor_schema::libraries;
use jiff::Timestamp;
use macro_attributes::insert;
use non_empty::{NonEmptyString, NonEmptyVec};
use ranged::{RangedU16, RangedU32};
use uuid::Uuid;

use crate::nucleic_acid::library::common::LibraryFields;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = libraries))]
pub struct LibraryCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: LibraryFields,
    number_of_sample_index_pcr_cycles: RangedU16<0, { u16::MAX }>,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    volume_µl: u8,
    target_reads_per_cell: RangedU32<0, { u32::MAX }>,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    prepared_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    preparer_ids: NonEmptyVec<Uuid, { usize::MAX }>,
}

impl LibraryCreation {
    #[must_use]
    pub fn cdna_id(&self) -> Uuid {
        self.inner.cdna_id
    }

    #[must_use]
    pub fn prepared_at(&self) -> Timestamp {
        self.prepared_at
    }

    #[must_use]
    pub fn single_index_set_name(&self) -> Option<&str> {
        self.inner
            .single_index_set_name
            .as_ref()
            .map(NonEmptyString::as_ref)
    }

    #[must_use]
    pub fn dual_index_set_name(&self) -> Option<&str> {
        self.inner
            .dual_index_set_name
            .as_ref()
            .map(NonEmptyString::as_ref)
    }

    #[must_use]
    pub fn volume_µl(&self) -> u8 {
        self.volume_µl
    }

    #[must_use]
    pub fn preparer_ids(&self) -> &[Uuid] {
        self.preparer_ids.as_ref()
    }
}
