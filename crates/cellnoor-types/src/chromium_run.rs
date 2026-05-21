use macro_attributes::{base_model, select};
use positive::PositiveF32;
pub use query::{
    ChromiumRunField, ChromiumRunPredicate, ChromiumRunPredicateInner, ChromiumRunQuery,
    SimpleChromiumRunQuery,
};

use crate::{
    chromium_run::record::{ChromiumRunRecord, GemWellRecord},
    id::{Id, NoId},
    simple_links::SimpleLinks,
    suspension_pool::{SavedTaggedSpecimenRecord, TaggedSpecimen},
    tenx_assay::TenxAssay,
    units::Microliter,
};

pub mod creation;
pub mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    use nonempty::NonemptyString;
    use serde_json::Value;
    use uuid::Uuid;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "gem_well"))]
    pub struct GemWellRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub readable_id: NonemptyString,
        pub chromium_run_id: Uuid,
    }

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "chromium_run"))]
    pub struct ChromiumRunRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub readable_id: NonemptyString,
        pub assay_id: Uuid,
        pub run_at: Timestamp,
        pub run_by: Uuid,
        pub succeeded: bool,
        pub additional_data: Option<Value>,
    }
}

pub type SavedGemWellRecord = GemWellRecord<Id>;

pub type NewChromiumRunRecord = ChromiumRunRecord<NoId>;

pub type SavedChromiumRunRecord = ChromiumRunRecord<Id>;

// The detailed view is read from two separate columns of `gem_well_to_specimen`
// and assembled in Rust, so this type is a plain serialization carrier — it
// intentionally does not derive `FromSql` (the `chromium_run_to_assay` view
// was removed).
pub type ChromiumRunUpdate = NewChromiumRunRecord;

#[base_model]
pub struct LoadingVolume {
    pub value: PositiveF32,
    pub unit: Microliter,
}

#[base_model]
pub struct ChromiumRunLinks {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: SimpleLinks,
    pub suspensions: String,
    pub suspension_pools: String,
}


#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "gem_well_with_specimens"))]
pub struct SavedGemWellWithSpecimensRecord {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub gem_well: SavedGemWellRecord,
    pub specimens: Vec<SavedTaggedSpecimenRecord>,
}

#[base_model]
pub struct GemWell {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedGemWellRecord,
    pub specimens: Vec<TaggedSpecimen>,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "view"))]
pub enum ChromiumRun {
    Compact {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedChromiumRunRecord,
        links: ChromiumRunLinks,
    },
    Detailed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedChromiumRunRecord,
        assay: TenxAssay,
        gem_wells: Vec<GemWell>,
        links: ChromiumRunLinks,
    },
}

impl ChromiumRun {
    pub fn record(&self) -> &SavedChromiumRunRecord {
        match self {
            Self::Compact { record, .. } | Self::Detailed { record, .. } => record,
        }
    }
}
