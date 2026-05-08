use jiff::Timestamp;
use macro_attributes::{base_model, select, unit_enum};
use nonempty::{NonemptyString, NonemptyVec};
pub use query::{
    SimpleSuspensionQuery, SuspensionPredicate, SuspensionPredicateInner, SuspensionQuery,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    simple_links::SimpleLinks,
    specimen::{Specimen, SpecimenRecord},
    suspension::measurement::SuspensionMeasurement,
};

pub mod measurement;
mod query;

#[unit_enum]
pub enum SuspensionContent {
    Cells,
    Nuclei,
}

mod new_suspension {
    use jiff::Timestamp;
    use macro_attributes::base_model;
    use nonempty::NonemptyString;
    use serde_json::Value;
    use uuid::Uuid;

    use crate::suspension::{SuspensionContent, measurement::NewSuspensionMeasurement};

    #[base_model]
    pub struct NewSuspension<P> {
        pub readable_id: NonemptyString,
        pub specimen_id: Uuid,
        pub content: SuspensionContent,
        pub created_at: Option<Timestamp>,
        pub lysis_duration_minutes: Option<f32>,
        pub target_cell_recovery: Option<i64>,
        pub additional_data: Option<Value>,
        #[cfg_attr(feature = "serde", serde(default))]
        pub measurements: Vec<NewSuspensionMeasurement>,
        pub preparers: P,
    }
}

pub type NewSuspension = new_suspension::NewSuspension<NonemptyVec<Uuid>>;

pub type SuspensionUpdate = new_suspension::NewSuspension<Option<NonemptyVec<Uuid>>>;

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "suspension"))]
pub struct SuspensionRecord {
    pub id: Uuid,
    pub readable_id: NonemptyString,
    pub specimen_id: Uuid,
    pub content: SuspensionContent,
    pub created_at: Option<Timestamp>,
    pub lysis_duration_minutes: Option<f32>,
    pub target_cell_recovery: Option<i64>,
    pub additional_data: Option<Value>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "suspension_detailed"))]
pub struct SuspensionRecordDetailed {
    pub suspension: SuspensionRecord,
    pub specimen: SpecimenRecord,
    pub measurements: Vec<SuspensionMeasurement>,
    pub preparers: Vec<Uuid>,
}

#[base_model]
pub enum Suspension {
    Compact {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SuspensionRecord,
        links: SimpleLinks,
    },
    // Rather than just wrapping the `SuspensionRecordDetailed`, we destructure its fields so that
    // we have a `Specimen` rather than a `SpecimenRecord`
    Detailed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SuspensionRecord,
        links: SimpleLinks,
        specimen: Specimen,
        measurements: Vec<SuspensionMeasurement>,
        preparers: Vec<Uuid>,
    },
}

impl SimpleLinks {
    fn for_suspension(id: Uuid) -> Self {
        Self::from_str_and_id("/suspensions", id)
    }
}

impl Suspension {
    pub fn record(&self) -> &SuspensionRecord {
        match self {
            Self::Compact { record, .. } => record,
            Self::Detailed { record, .. } => record,
        }
    }

    pub fn from_record(record: SuspensionRecord) -> Self {
        Self::Compact {
            links: SimpleLinks::for_suspension(record.id),
            record,
        }
    }

    pub fn from_detailed_record(
        SuspensionRecordDetailed {
            suspension,
            specimen,
            measurements,
            preparers,
        }: SuspensionRecordDetailed,
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
