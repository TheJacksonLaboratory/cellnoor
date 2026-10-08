use macro_attributes::{base_model, select, unit_enum};
pub use query::{
    MultiplexingTagField, MultiplexingTagPredicate, MultiplexingTagTypeOperator,
    SimpleSuspensionPoolQuery, SuspensionPoolField, SuspensionPoolPredicate,
    SuspensionPoolPredicateInner, SuspensionPoolQuery,
};
use uuid::Uuid;

use crate::{
    Relation,
    chromium_run::creation::ocm::OcmBarcodeId,
    id::{Id, NoId},
    multiplexing_tag::MultiplexingTag,
    nonempty::{NonemptyString, NonemptyVec},
    simple_links::SimpleLinks,
    specimen::{SavedSpecimenRecord, SpecimenCompact},
    suspension_pool::{measurement::SuspensionPoolMeasurement, record::SuspensionPoolRecord},
};

pub mod measurement;
mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    use serde_json::Value;

    use crate::nonempty::NonemptyString;

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

impl<T> Relation for SuspensionPoolRecord<T> {
    const NAME: &'static str = "suspension_pool";
}

pub type NewSuspensionPoolRecord = SuspensionPoolRecord<NoId>;

#[unit_enum]
#[derive(strum::VariantArray)]
pub enum MultiplexingTagType {
    FlexBarcode,
    FlexOligonucleotideBarcode,
    #[cfg_attr(feature = "serde", serde(rename = "TotalSeq-A"))]
    #[strum(serialize = "TotalSeq-A")]
    TotalSeqA,
    #[cfg_attr(feature = "serde", serde(rename = "TotalSeq-B"))]
    #[strum(serialize = "TotalSeq-B")]
    TotalSeqB,
    #[cfg_attr(feature = "serde", serde(rename = "TotalSeq-C"))]
    #[strum(serialize = "TotalSeq-C")]
    TotalSeqC,
}

#[base_model]
pub struct TaggedSuspension {
    pub suspension_id: Uuid,
    pub tag_id: NonemptyString,
}

#[base_model]
pub struct NewSuspensionPool {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewSuspensionPoolRecord,
    pub measurements: Vec<measurement::NewSuspensionPoolMeasurement>,
    pub preparers: NonemptyVec<Uuid>,
    pub pool: PooledSuspensions,
}

#[base_model]
#[cfg_attr(
    feature = "serde",
    serde(untagged, deny_unknown_fields, rename_all = "snake_case")
)]
pub enum PooledSuspensions {
    ExogenouslyTagged {
        suspensions: NonemptyVec<TaggedSuspension>,
        multiplexing_tag_type: MultiplexingTagType,
    },
    GeneticallyTagged {
        suspensions: NonemptyVec<Uuid>,
    },
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

#[cfg(all(test, feature = "serde"))]
mod tests {
    use jiff::Timestamp;
    use strum::VariantArray;
    use uuid::Uuid;

    use crate::suspension_pool::{MultiplexingTagType, NewSuspensionPool};

    fn pool() -> serde_json::Value {
        serde_json::json!(
            {
                "readable_id": "id",
                "name": "name",
                "pooled_at": Timestamp::now(),
                "measurements": [],
                "preparers": [Uuid::nil()],
                "pool": {
                    "suspensions": [
                        {
                            "suspension_id": Uuid::nil(),
                            "tag_id": "tag"
                        }
                    ],
                    "multiplexing_tag_type": MultiplexingTagType::FlexOligonucleotideBarcode
                },
            }
        )
    }

    #[test]
    fn suspension_pool_deserializes_correctly() {
        serde_json::from_value::<NewSuspensionPool>(pool()).unwrap();
    }

    #[test]
    fn multiplexing_tag_type_strum_and_serde_match() {
        for ty in MultiplexingTagType::VARIANTS {
            let serde = serde_json::to_string(ty).unwrap();
            let strum: &str = ty.as_ref();

            // Wrap the strum serialization in double quotes because it's
            // supposed to be JSON
            pretty_assertions::assert_eq!(serde, format!(r#""{strum}""#));
        }
    }

    #[test]
    fn suspension_pool_rejects_unknown_fields() {
        let mut pool = pool();
        pool["pool"]["foo"] = serde_json::Value::String("bar".to_owned());

        serde_json::from_value::<NewSuspensionPool>(pool).unwrap_err();
    }
}
