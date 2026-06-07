use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::{NonemptyBoundedVec, NonemptyString, NonemptyVec};
pub use query::{
    SimpleSuspensionPoolQuery, SuspensionPoolField, SuspensionPoolPredicate,
    SuspensionPoolPredicateInner, SuspensionPoolQuery,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    chromium_run::creation::ocm::OcmBarcodeId,
    multiplexing_tag::MultiplexingTag,
    simple_links::SimpleLinks,
    specimen::{SavedSpecimenRecord, SpecimenCompact},
    suspension_pool::measurement::SuspensionPoolMeasurement,
};

pub mod measurement;
mod query;

#[base_model]
pub struct NewSuspensionPoolCommonFields {
    pub readable_id: NonemptyString,
    pub name: NonemptyString,
    pub pooled_at: Timestamp,
    pub additional_data: Option<Value>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "suspension_pool"))]
pub struct SavedSuspensionPoolRecord {
    pub id: Uuid,
    pub readable_id: NonemptyString,
    pub name: NonemptyString,
    pub multiplexing_type: NonemptyString,
    pub pooled_at: Timestamp,
    pub additional_data: Option<Value>,
}

#[base_model]
pub struct TaggedSuspension {
    pub suspension_id: Uuid,
    pub tag_id: Uuid,
}

// https://www.10xgenomics.com/products/flex-gene-expression
const MAX_TAGGED_SUSPENSIONS_IN_POOL: usize = 384;

#[base_model]
#[derive(strum::AsRefStr)]
#[cfg_attr(feature = "serde", serde(untagged, rename_all = "snake_case"))]
#[strum(serialize_all = "snake_case")]
pub enum NewSuspensionPool {
    ExogenousTag {
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewSuspensionPoolCommonFields,
        measurements: Vec<measurement::NewSuspensionPoolMeasurement>,
        preparers: NonemptyVec<Uuid>,
        suspensions: NonemptyBoundedVec<TaggedSuspension, MAX_TAGGED_SUSPENSIONS_IN_POOL>,
    },
    Genetic {
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewSuspensionPoolCommonFields,
        measurements: Vec<measurement::NewSuspensionPoolMeasurement>,
        preparers: NonemptyVec<Uuid>,
        suspensions: NonemptyVec<Uuid>,
    },
}

#[base_model]
pub struct SuspensionPoolUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewSuspensionPoolCommonFields,
    pub measurements: Option<Vec<measurement::NewSuspensionPoolMeasurement>>,
    pub preparers: Option<Vec<Uuid>>,
}

#[base_model]
pub struct SuspensionPoolLinks {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: SimpleLinks,
    pub suspensions: String,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "tagged_specimen"))]
pub struct SavedTaggedSpecimenRecord {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub specimen: SavedSpecimenRecord,
    pub multiplexing_tag: Option<MultiplexingTag>,
    pub ocm_barcode_id: Option<OcmBarcodeId>,
}

#[base_model]
pub struct TaggedSpecimen {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub specimen: SpecimenCompact,
    pub multiplexing_tag: Option<MultiplexingTag>,
    pub ocm_barcode_id: Option<OcmBarcodeId>,
}

#[base_model]
pub struct SuspensionPoolCompact {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedSuspensionPoolRecord,
    pub links: SuspensionPoolLinks,
}

#[base_model]
pub struct SuspensionPoolDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedSuspensionPoolRecord,
    pub links: SuspensionPoolLinks,
    pub specimens: Vec<TaggedSpecimen>,
    pub measurements: Vec<SuspensionPoolMeasurement>,
    pub preparers: Vec<Uuid>,
}
