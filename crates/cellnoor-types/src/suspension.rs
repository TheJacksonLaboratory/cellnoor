use macro_attributes::{base_model, select, unit_enum};
use nonempty::NonemptyVec;
pub use query::{
    SimpleSuspensionQuery, SuspensionField, SuspensionPredicate, SuspensionPredicateInner,
    SuspensionQuery,
};
use uuid::Uuid;

use crate::{
    id::{Id, NoId},
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
    use nonempty::NonemptyString;
    use serde_json::Value;
    use uuid::Uuid;

    use crate::suspension::SuspensionContent;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "suspension"))]
    pub struct SuspensionRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub readable_id: NonemptyString,
        pub specimen_id: Uuid,
        pub content: SuspensionContent,
        pub created_at: Option<Timestamp>,
        pub lysis_duration_minutes: Option<f32>,
        pub target_cell_recovery: Option<i64>,
        pub additional_data: Option<Value>,
    }
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
    #[cfg_attr(feature = "serde", serde(default))]
    pub measurements: Vec<NewSuspensionMeasurement>,
    pub preparers: NonemptyVec<Uuid>,
}

#[base_model]
pub struct SuspensionUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewSuspensionRecord,
    #[cfg_attr(feature = "serde", serde(default))]
    pub measurements: Vec<NewSuspensionMeasurement>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub preparers: Vec<Uuid>,
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
