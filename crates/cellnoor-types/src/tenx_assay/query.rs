use macro_attributes::{predicate_enum, sort_field_enum};

use crate::operator::{StringOperator, UuidOperator};

#[predicate_enum]
#[strum(prefix = "(tenx_assay).")]
#[strum_discriminants(name(TenxAssayField), sort_field_enum, strum(prefix = "(tenx_assay)."))]
pub enum TenxAssayPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    ChemistryVersion(StringOperator),
    ProtocolUrl(StringOperator),
}

#[cfg(feature = "postgres-types")]

impl Default for TenxAssayField {
    fn default() -> Self {
        Self::Name
    }
}
