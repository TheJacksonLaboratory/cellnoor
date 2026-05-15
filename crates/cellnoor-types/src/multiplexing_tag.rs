use macro_attributes::unit_enum;

use crate::{
    id::{Id, NoId},
    multiplexing_tag::record::MultiplexingTagRecord,
};

mod record {
    use macro_attributes::select;
    use nonempty::NonemptyString;

    use crate::multiplexing_tag::MultiplexingTagType;

    #[select]
    #[cfg_attr(feature = "schemars", schemars(inline))]
    #[cfg_attr(feature = "postgres-types", postgres(name = "multiplexing_tag"))]
    pub struct MultiplexingTagRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub tag_id: NonemptyString,
        #[cfg_attr(feature = "postgres-types", postgres(name = "type"))]
        pub type_: MultiplexingTagType,
    }
}

pub type NewMultiplexingTag = MultiplexingTagRecord<NoId>;

pub type SavedMultiplexingTag = MultiplexingTagRecord<Id>;

#[unit_enum]
pub enum MultiplexingTagType {
    FlexBarcode,
    FlexOligonucleotideBarcode,
    OnChipMultiplexing,
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
