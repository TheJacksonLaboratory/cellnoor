use macro_attributes::{base_model, select};
use positive::PositiveF32;

use crate::{
    chromium_run::record::{ChromiumRunRecord, GemPoolRecord},
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
    #[cfg_attr(feature = "postgres-types", postgres(name = "gem_pool"))]
    pub struct GemPoolRecord<T> {
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

pub type SavedGemPoolRecord = GemPoolRecord<Id>;

pub type NewChromiumRunRecord = ChromiumRunRecord<NoId>;

pub type SavedChromiumRunRecord = ChromiumRunRecord<Id>;

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

impl ChromiumRunLinks {
    fn from_id(id: Id) -> Self {
        Self {
            simple: SimpleLinks::from_str_and_id("/chromium-runs", id),
            suspensions: format!("/chromium-runs/{id}/suspensions"),
            suspension_pools: format!("/chromium-runs/{id}/suspension-pools"),
        }
    }
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "gem_pool_with_specimens"))]
pub struct GemPoolWithSpecimensRecord {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub gem_pool: SavedGemPoolRecord,
    pub specimens: Vec<SavedTaggedSpecimenRecord>,
}

#[base_model]
pub struct GemPool {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub gem_pool: SavedGemPoolRecord,
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
        gem_pools: Vec<GemPool>,
        links: ChromiumRunLinks,
    },
}

impl ChromiumRun {
    pub fn from_record(record: SavedChromiumRunRecord) -> Self {
        Self::Compact {
            links: ChromiumRunLinks::from_id(record.id),
            record,
        }
    }
}
