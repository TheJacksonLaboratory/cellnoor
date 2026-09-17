use macro_attributes::{base_model, select, unit_enum};
pub use query::{
    SimpleSuspensionQuery, SuspensionContentOperator, SuspensionField, SuspensionPredicate,
    SuspensionPredicateInner, SuspensionQuery,
};
use uuid::Uuid;

use crate::{
    Relation,
    id::{Id, NoId},
    nonempty::NonemptyVec,
    simple_links::SimpleLinks,
    specimen::{SavedSpecimenRecord, SpecimenCompact},
    suspension::{
        measurement::{NewSuspensionMeasurement, SuspensionMeasurement},
        record::SuspensionRecord,
    },
};

pub mod measurement;
mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{nonempty::NonemptyString, suspension::SuspensionContent};

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "suspension"))]
    pub struct SuspensionRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub readable_id: NonemptyString,
        pub specimen_id: Uuid,
        #[cfg_attr(feature = "serde", serde(skip))]
        pub specimen_received_at: Timestamp,
        pub content: SuspensionContent,
        pub created_at: Option<Timestamp>,
        pub lysis_duration_minutes: Option<f32>,
        pub target_cell_recovery: Option<i64>,
        pub additional_data: Option<Value>,
    }
}

impl<T> Relation for SuspensionRecord<T> {
    const NAME: &'static str = "suspension";
}

pub type NewSuspensionRecord = SuspensionRecord<NoId>;

pub type SavedSuspensionRecord = SuspensionRecord<Id>;

#[unit_enum]
pub enum SuspensionContent {
    Cells,
    Nuclei,
}

#[base_model]
pub struct NewSuspension {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewSuspensionRecord,
    pub measurements: Vec<NewSuspensionMeasurement>,
    pub preparers: NonemptyVec<Uuid>,
}

#[base_model]
pub struct SuspensionUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewSuspensionRecord,
    pub measurements: Option<Vec<NewSuspensionMeasurement>>,
    pub preparers: Option<Vec<Uuid>>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "suspension_detailed"))]
pub struct SavedSuspensionRecordDetailed {
    pub suspension: SavedSuspensionRecord,
    pub specimen: SavedSpecimenRecord,
    pub measurements: Vec<SuspensionMeasurement>,
    pub preparers: Vec<Uuid>,
}

#[base_model]
pub struct SuspensionCompact {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedSuspensionRecord,
    pub links: SimpleLinks,
}

// Rather than just wrapping `SavedSuspensionRecordDetailed`, we destructure its
// fields so that we have a `SpecimenCompact` rather than a bare
// `SavedSpecimenRecord`.
#[base_model]
pub struct SuspensionDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedSuspensionRecord,
    pub links: SimpleLinks,
    pub specimen: SpecimenCompact,
    pub measurements: Vec<SuspensionMeasurement>,
    pub preparers: Vec<Uuid>,
}
