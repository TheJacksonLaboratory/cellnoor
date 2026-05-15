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
    specimen::{SavedSpecimenRecord, Specimen},
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
#[cfg_attr(feature = "serde", serde(tag = "view"))]
pub enum Suspension {
    Compact {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedSuspensionRecord,
        links: SimpleLinks,
    },
    // Rather than just wrapping the `SuspensionRecordDetailed`, we destructure its fields so that
    // we have a `Specimen` rather than a `SpecimenRecord`
    Detailed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedSuspensionRecord,
        links: SimpleLinks,
        specimen: Specimen,
        measurements: Vec<SuspensionMeasurement>,
        preparers: Vec<Uuid>,
    },
}

impl SimpleLinks {
    fn for_suspension(id: Id) -> Self {
        Self::from_str_and_id("/suspensions", id)
    }
}

impl Suspension {
    pub fn record(&self) -> &SavedSuspensionRecord {
        match self {
            Self::Compact { record, .. } => record,
            Self::Detailed { record, .. } => record,
        }
    }

    pub fn from_record(record: SavedSuspensionRecord) -> Self {
        Self::Compact {
            links: SimpleLinks::for_suspension(record.id),
            record,
        }
    }

    pub fn from_detailed_record(
        SavedSuspensionRecordDetailed {
            suspension,
            specimen,
            measurements,
            preparers,
        }: SavedSuspensionRecordDetailed,
    ) -> Self {
        Self::Detailed {
            links: SimpleLinks::for_suspension(suspension.id),
            record: suspension,
            specimen: Specimen::from_record(specimen),
            measurements,
            preparers,
        }
    }
}
