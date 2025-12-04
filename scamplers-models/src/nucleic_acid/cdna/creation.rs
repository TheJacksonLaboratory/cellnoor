use jiff::Timestamp;
use macro_attributes::insert;
use non_empty::NonEmptyVec;
use positive::PositiveU32;
#[cfg(feature = "app")]
use scamplers_schema::cdna;
use uuid::Uuid;

use crate::{nucleic_acid::cdna::common::CdnaFields, tenx_assay::LibraryType};

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = cdna))]
pub struct CdnaCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: CdnaFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp, deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    prepared_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    volume_µl: PositiveU32,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    preparer_ids: NonEmptyVec<Uuid>,
}

impl CdnaCreation {
    #[must_use]
    pub fn assay_id(&self) -> Uuid {
        self.inner.assay_id
    }

    #[must_use]
    pub fn library_type(&self) -> LibraryType {
        self.inner.library_type
    }

    #[must_use]
    pub fn volume_µl(&self) -> PositiveU32 {
        self.volume_µl
    }
}
