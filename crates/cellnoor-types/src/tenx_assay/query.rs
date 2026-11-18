use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    Relation,
    operator::{StringOperator, UuidOperator},
    query::{Field, OrderField, filter::Operator},
    tenx_assay::{SampleMultiplexing, TenxAssay},
};

pub type SampleMultiplexingOperator = Operator<SampleMultiplexing>;

#[predicate_enum(TenxAssayField)]
pub enum TenxAssayPredicate {
    Id(UuidOperator),
    Name(StringOperator),
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
