use macro_attributes::{base_model, select, sort_field_enum};
use nonempty::{NonemptyString, NonemptyVec};
#[cfg(feature = "postgres-types")]
use postgres_types::{FromSql, ToSql, to_sql_checked};
pub use query::{
    MultiplexingTagField, MultiplexingTagPredicate, SimpleSuspensionPoolQuery, SuspensionPoolField,
    SuspensionPoolPredicate, SuspensionPoolPredicateInner, SuspensionPoolQuery,
};
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
    serde(tag = "multiplexing_tag_type", rename_all = "snake_case")
)]
#[strum(serialize_all = "snake_case")]
// Apply `sort_field_enum` cus it has everything we want
#[strum_discriminants(name(MultiplexingTagType), sort_field_enum)]
pub enum NewSuspensionPool {
    FlexBarcode(NewTaggedSuspensionPool),
    FlexOligonucleotideBarcode(NewTaggedSuspensionPool),
    #[cfg_attr(feature = "serde", serde(rename = "TotalSeq-A"))]
    #[strum(serialize = "TotalSeq-A")]
    #[strum_discriminants(cfg_attr(feature = "serde", serde(rename = "TotalSeq-A")))]
    #[strum_discriminants(strum(serialize = "TotalSeq-A"))]
    TotalSeqA(NewTaggedSuspensionPool),
    #[cfg_attr(feature = "serde", serde(rename = "TotalSeq-B"))]
    #[strum(serialize = "TotalSeq-B")]
    #[strum_discriminants(cfg_attr(feature = "serde", serde(rename = "TotalSeq-B")))]
    #[strum_discriminants(strum(serialize = "TotalSeq-B"))]
    TotalSeqB(NewTaggedSuspensionPool),
    #[cfg_attr(feature = "serde", serde(rename = "TotalSeq-C"))]
    #[strum(serialize = "TotalSeq-C")]
    #[strum_discriminants(cfg_attr(feature = "serde", serde(rename = "TotalSeq-C")))]
    #[strum_discriminants(strum(serialize = "TotalSeq-C"))]
    TotalSeqC(NewTaggedSuspensionPool),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(disabled)]
    #[strum_discriminants(strum(disabled))]
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

        NonemptyString::from_sql(ty, raw).map(|s| Self::from_str(s.as_ref()).unwrap())
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        <NonemptyString as FromSql>::accepts(ty)
    }
}

#[cfg(feature = "postgres-types")]
impl ToSql for MultiplexingTagType {
    to_sql_checked!();

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
    use crate::suspension_pool::{MultiplexingTagType, NewSuspensionPool};
    use strum::VariantArray;

    // These tests cover 3 axes:
    // 1. The serde (de)serializtion of MultiplexingTagType matches its strum (de)serialization
    // 2. MultiplexingTagType can round-trip as a string
    // 3. The serde (de)serializtion of NewSuspensionPool matches that of MultiplexingTagType
    #[cfg(feature = "serde")]
    #[test]
    fn suspension_pool_type_matches_multiplexing_tag_type() {
        for ty in MultiplexingTagType::VARIANTS {
            use jiff::Timestamp;
            use uuid::Uuid;

            if matches!(ty, MultiplexingTagType::Genetic) {
                continue;
            }

            // First, we construct a pool using the serde serialization of MultiplexingTagType
            let mut pool = serde_json::json!(
                {
                    "readable_id": "id",
                    "name": "name",
                    "pooled_at": Timestamp::now(),
                    "measurements": [],
                    "preparers": [Uuid::nil()],
                    "suspensions": [
                        {
                            "suspension_id": Uuid::nil(),
                            "tag_id": "tag"
                        }
                    ],
                    "multiplexing_tag_type": ty
                }
            );
            let Ok(deserialized_pool) = serde_json::from_value::<NewSuspensionPool>(pool.clone())
            else {
                panic!("failed to deserialize the following JSON: as NewSuspensionPool:\n{pool}");
            };

            // Next, ensure that the strum serializations of the two types match
            let pool_as_str: &str = deserialized_pool.clone().into();
            let ty_as_str: &str = ty.into();
            pretty_assertions::assert_str_eq!(pool_as_str, ty_as_str);

            // Finally, ensure that the strum serialization of MultiplexingTagType yields the same result as the serde serialization
            pool["multiplexing_tag_type"] = serde_json::Value::String(pool_as_str.to_owned());
            pretty_assertions::assert_eq!(deserialized_pool, serde_json::from_value(pool).unwrap());
        }
    }
}
