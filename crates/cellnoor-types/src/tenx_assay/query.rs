use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{StringOperator, UuidOperator},
    query::filter::{ArrayOperator, Operator},
    tenx_assay::{LibraryType, SampleMultiplexing},
};

pub type SampleMultiplexingOperator = Operator<SampleMultiplexing>;

#[predicate_enum]
#[strum(prefix = "(tenx_assay).")]
#[strum_discriminants(name(TenxAssayField), sort_field_enum, strum(prefix = "(tenx_assay)."))]
pub enum TenxAssayPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    LibraryTypes(ArrayOperator<LibraryType>),
    SampleMultiplexing(SampleMultiplexingOperator),
    ChemistryVersion(StringOperator),
    ChromiumChip(StringOperator),
    ProtocolUrl(StringOperator),
}

#[cfg(feature = "postgres-types")]

impl Default for TenxAssayField {
    fn default() -> Self {
        Self::Name
    }
}
