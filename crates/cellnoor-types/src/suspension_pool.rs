use jiff::Timestamp;
use macro_attributes::{base_model, select, sort_field_enum};
use nonempty::{NonemptyBoundedVec, NonemptyString, NonemptyVec};
#[cfg(feature = "postgres-types")]
use postgres_types::{FromSql, ToSql, to_sql_checked};
pub use query::{
    MultiplexingTagField, MultiplexingTagPredicate, SimpleSuspensionPoolQuery, SuspensionPoolField,
    SuspensionPoolPredicate, SuspensionPoolPredicateInner, SuspensionPoolQuery,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    chromium_run::creation::ocm::OcmBarcodeId,
    id::{Id, NoId},
    multiplexing_tag::MultiplexingTag,
    simple_links::SimpleLinks,
    specimen::{SavedSpecimenRecord, SpecimenCompact},
    suspension_pool::{measurement::SuspensionPoolMeasurement, record::SuspensionPoolRecord},
};

pub mod measurement;
mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    use nonempty::NonemptyString;
    use serde_json::Value;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "suspension_pool"))]
    pub struct SuspensionPoolRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub readable_id: NonemptyString,
        pub name: NonemptyString,
        pub pooled_at: Timestamp,
        pub additional_data: Option<Value>,
    }
}

pub type NewSuspensionPoolRecord = SuspensionPoolRecord<NoId>;

#[base_model]
pub struct NewSuspensionPoolCommonFields {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewSuspensionPoolRecord,
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
#[derive(strum::IntoStaticStr, strum::EnumDiscriminants)]
#[cfg_attr(
    feature = "serde",
    serde(tag = "multiplexing_type", rename_all = "snake_case")
)]
#[strum(serialize_all = "snake_case")]
// Apply `sort_field_enum` cus it has everything we want
#[strum_discriminants(name(MultiplexingTagType), sort_field_enum)]
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
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(disabled)]
    Genetic {
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewSuspensionPoolCommonFields,
        suspensions: NonemptyVec<Uuid>,
    },
}

#[cfg(feature = "postgres-types")]
impl<'a> FromSql<'a> for MultiplexingTagType {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        use std::str::FromStr;

        let string = NonemptyString::from_sql(ty, raw).unwrap();
        dbg!(string);
        let as_enum: &str = MultiplexingTagType::FlexBarcode.into();
        dbg!(as_enum);

        NonemptyString::from_sql(ty, raw).map(|s| Self::from_str(s.as_ref()).unwrap())
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        <NonemptyString as FromSql>::accepts(ty)
    }
}

#[cfg(feature = "postgres-types")]
impl ToSql for MultiplexingTagType {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let as_str: &str = self.into();

        NonemptyString::new(as_str.to_owned())
            .unwrap()
            .to_sql(ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        <NonemptyString as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

#[base_model]
pub struct SuspensionPoolUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewSuspensionPoolRecord,
    pub measurements: Option<Vec<measurement::NewSuspensionPoolMeasurement>>,
    pub preparers: Option<Vec<Uuid>>,
}

pub type SavedSuspensionPoolRecord = SuspensionPoolRecord<Id>;

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

#[cfg(test)]
mod tests {
    use crate::suspension_pool::MultiplexingTagType;
    use std::str::FromStr;
    use strum::VariantArray;

    #[test]
    fn multiplexing_tag_type_serialization() {
        for ty in MultiplexingTagType::VARIANTS {
            MultiplexingTagType::from_str(ty.into()).unwrap();
        }
    }
}
