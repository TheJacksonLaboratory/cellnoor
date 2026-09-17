use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    Relation,
    operator::{StringOperator, UuidOperator},
    query::{
        Field, OrderField,
        filter::{ArrayOperator, Operator},
    },
    tenx_assay::{LibraryType, SampleMultiplexing, TenxAssay},
};

pub type SampleMultiplexingOperator = Operator<SampleMultiplexing>;

#[predicate_enum(TenxAssayField)]
pub enum TenxAssayPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    LibraryTypes(ArrayOperator<LibraryType>),
    SampleMultiplexing(SampleMultiplexingOperator),
    ChemistryVersion(StringOperator),
    ChromiumChip(StringOperator),
    ProtocolUrl(StringOperator),
}

impl Field for TenxAssayField {
    const RELATION: &'static str = <TenxAssay as Relation>::NAME;
}

impl OrderField for TenxAssayField {
    fn default_field() -> Self {
        Self::Name
    }

    fn default_desc() -> bool {
        false
    }
}
