use macro_attributes::{predicate_enum, sort_field_enum};
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

use crate::operator::{StringOperator, UuidOperator};
#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;

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
impl ToPredicate for TenxAssayPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Id(u) => u.to_predicate(),
            Self::Name(s) | Self::ChemistryVersion(s) | Self::ProtocolUrl(s) => s.to_predicate(),
        }
    }
}

impl Default for TenxAssayField {
    fn default() -> Self {
        Self::Name
    }
}
