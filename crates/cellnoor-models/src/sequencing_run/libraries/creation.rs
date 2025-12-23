#[cfg(feature = "app")]
use cellnoor_schema::sequencing_submissions;
use jiff::Timestamp;
use macro_attributes::insert;
use uuid::Uuid;

#[insert]
pub struct SequencingSubmission {
    library_id: Uuid,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    submitted_at: Timestamp,
}

impl SequencingSubmission {
    #[must_use]
    pub fn library_id(&self) -> Uuid {
        self.library_id
    }

    #[must_use]
    pub fn submitted_at(&self) -> Timestamp {
        self.submitted_at
    }
}
