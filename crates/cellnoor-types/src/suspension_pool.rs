use jiff::Timestamp;
use macro_attributes::{base_model, select, sort_field_enum};
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
    pub measurements: Vec<measurement::NewSuspensionPoolMeasurement>,
    pub preparers: NonemptyVec<Uuid>,
}

#[base_model]
pub struct TaggedSuspension {
    pub suspension_id: Uuid,
    pub tag_id: NonemptyString,
}

#[base_model]
pub struct NewTaggedSuspensionPool {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub common: NewSuspensionPoolCommonFields,
    pub suspensions: NonemptyVec<TaggedSuspension>,
}

#[base_model]
#[derive(strum::AsRefStr, strum::EnumDiscriminants)]
#[cfg_attr(
    feature = "serde",
    serde(tag = "multiplexing_type", rename_all = "snake_case")
)]
#[strum(serialize_all = "snake_case")]
#[strum_discriminants(name(MultiplexingTagType), sort_field_enum, derive(strum::EnumString))]
pub enum NewSuspensionPool {
    FlexBarcode(NewTaggedSuspensionPool),
    FlexOligoNucleotideBarcode(NewTaggedSuspensionPool),
    #[cfg_attr(feature = "serde", serde(rename = "TotalSeq-A"))]
    #[strum(serialize = "TotalSeq-A")]
    TotalSeqA(NewTaggedSuspensionPool),
    #[cfg_attr(feature = "serde", serde(rename = "TotalSeq-B"))]
    #[strum(serialize = "TotalSeq-B")]
    TotalSeqB(NewTaggedSuspensionPool),
    #[cfg_attr(feature = "serde", serde(rename = "TotalSeq-C"))]
    #[strum(serialize = "TotalSeq-C")]
    TotalSeqC(NewTaggedSuspensionPool),
    Genetic {
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewSuspensionPoolCommonFields,
        suspensions: NonemptyVec<Uuid>,
    },
}

#[cfg(feature = "postgres-types")]
impl<'a> postgres_types::FromSql<'a> for MultiplexingTagType {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        use std::str::FromStr;

        NonemptyString::from_sql(ty, raw).map(|s| Self::from_str(s.as_ref()).unwrap())
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        NonemptyString::accepts(ty)
    }
}

#[base_model]
pub struct SuspensionPoolUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewSuspensionPoolCommonFields,
    pub measurements: Option<Vec<measurement::NewSuspensionPoolMeasurement>>,
    pub preparers: Option<Vec<Uuid>>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "suspension_pool"))]
pub struct SavedSuspensionPoolRecord {
    pub id: Uuid,
    pub readable_id: NonemptyString,
    pub name: NonemptyString,
    pub multiplexing_type: MultiplexingTagType,
    pub pooled_at: Timestamp,
    pub additional_data: Option<Value>,
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
