use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::{NonemptyBoundedVec, NonemptyString, NonemptyVec};
use serde_json::Value;
use uuid::Uuid;

#[base_model]
pub struct TaggedSuspension {
    pub suspension_id: Uuid,
    pub tag_id: Uuid,
}

// https://www.10xgenomics.com/products/flex-gene-expression
const MAX_TAGGED_SUSPENSIONS_IN_POOL: usize = 384;

#[base_model]
pub struct NewSuspensionPoolFields {
    pub readable_id: NonemptyString,
    pub name: NonemptyString,
    pub pooled_at: Timestamp,
    pub additional_data: Option<Value>,
}

#[base_model]
#[derive(strum::AsRefStr)]
#[cfg_attr(
    feature = "serde",
    serde(tag = "multiplexing_type", rename_all = "snake_case")
)]
#[strum(serialize_all = "snake_case")]
pub enum NewSuspensionPool {
    ExogenousTag {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: NewSuspensionPoolFields,
        preparer_ids: NonemptyVec<Uuid>,
        suspensions: NonemptyBoundedVec<TaggedSuspension, MAX_TAGGED_SUSPENSIONS_IN_POOL>,
    },
    Genetic {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: NewSuspensionPoolFields,
        preparer_ids: NonemptyVec<Uuid>,
        suspensions: NonemptyVec<Uuid>,
    },
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "suspension_pool"))]
pub struct SuspensionPoolRecord {
    pub id: Uuid,
    pub readable_id: NonemptyString,
    pub name: NonemptyString,
    pub multiplexing_type: String,
    pub pooled_at: Timestamp,
    pub additional_data: Option<Value>,
}

// #[select]
// #[cfg_attr(feature = "postgres-types", postgres(name = "specimen_detailed"))]
// pub struct SuspensionPoolRecordDetailed {
//     #[cfg_attr(feature = "serde", serde(flatten))]
//     pub suspension_pool: SuspensionPoolRecord,
//     pub suspensions: Vec<Suspension>,
//     pub preparers: Vec<Uuid>,
//     pub measurements: Vec<SuspensionPoolMeasurement>,
// }
