use macro_attributes::{base_model, select};
pub use query::{
    ChromiumRunField, ChromiumRunPredicate, ChromiumRunPredicateInner, ChromiumRunQuery,
    SimpleChromiumRunQuery,
};

use crate::{
    Relation,
    chromium_run::{
        creation::NewChromiumRunRecord,
        record::{ChromiumRunRecord, GemWellRecord},
    },
    id::Id,
    simple_links::SimpleLinks,
    suspension_pool::{SavedTaggedSpecimenRecord, TaggedSpecimen},
    tenx_assay::TenxAssay,
};

pub mod creation;
pub mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{id::Id, nonempty::NonemptyString};

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "gem_well"))]
    pub struct GemWellRecord {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: Id,
        pub readable_id: NonemptyString,
        pub chromium_run_id: Uuid,
        #[cfg_attr(feature = "serde", serde(skip))]
        pub run_at: Timestamp,
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

impl Relation for GemWellRecord {
    const NAME: &'static str = "gem_well";
}

impl<T> Relation for ChromiumRunRecord<T> {
    const NAME: &'static str = "chromium_run";
}

pub type SavedChromiumRunRecord = ChromiumRunRecord<Id>;

pub type ChromiumRunUpdate = NewChromiumRunRecord;

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
    pub gem_well: GemWellRecord,
    pub specimens: Vec<SavedTaggedSpecimenRecord>,
}

#[base_model]
pub struct GemWell {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: GemWellRecord,
    pub specimens: Vec<TaggedSpecimen>,
}

#[base_model]
pub struct ChromiumRunCompact {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedChromiumRunRecord,
    pub links: ChromiumRunLinks,
}

#[base_model]
pub struct ChromiumRunDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedChromiumRunRecord,
    pub assay: TenxAssay,
    pub gem_wells: Vec<GemWell>,
    pub links: ChromiumRunLinks,
}
