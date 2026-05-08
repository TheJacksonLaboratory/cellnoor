use macro_attributes::{predicate_enum, sort_field_enum};
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;
use crate::{
    StringOperator, TimestampOperator, UuidOperator,
    query::{ComplexQuery, SimpleQuery, filter::Operator},
    specimen::{
        Fixative, Species, SpecimenType, ThermalPreservationMethod,
        creation::block::BlockEmbeddingMatrix,
    },
};

pub type SpeciesOperator = Operator<Species>;
pub type SpecimenTypeOperator = Operator<SpecimenType>;
pub type BlockEmbeddingMatrixOperator = Operator<BlockEmbeddingMatrix>;
pub type FixativeOperator = Operator<Fixative>;
pub type ThermalPreservationMethodOperator = Operator<ThermalPreservationMethod>;

#[predicate_enum]
#[strum(prefix = "(specimen).")]
#[strum_discriminants(name(SpecimenField), sort_field_enum, strum(prefix = "(specimen)."))]
pub enum SpecimenPredicate {
    Id(UuidOperator),
    ReadableId(StringOperator),
    Name(StringOperator),
    SubmittedBy(UuidOperator),
    ProjectId(UuidOperator),
    ReceivedAt(TimestampOperator),
    Species(SpeciesOperator),
    HostSpecies(SpeciesOperator),
    ReturnedAt(TimestampOperator),
    ReturnedBy(UuidOperator),
    Type(SpecimenTypeOperator),
    EmbeddedIn(BlockEmbeddingMatrixOperator),
    Fixative(FixativeOperator),
    ThermalPreservationMethod(ThermalPreservationMethodOperator),
    Tissue(StringOperator),
}

#[cfg(feature = "postgres-types")]
impl ToPredicate for SpecimenPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Id(u) | Self::SubmittedBy(u) | Self::ProjectId(u) | Self::ReturnedBy(u) => {
                u.to_predicate()
            }
            Self::ReadableId(s) | Self::Name(s) | Self::Tissue(s) => s.to_predicate(),
            Self::ReceivedAt(t) | Self::ReturnedAt(t) => t.to_predicate(),
            Self::Species(sp) | Self::HostSpecies(sp) => sp.to_predicate(),
            Self::Type(ty) => ty.to_predicate(),
            Self::EmbeddedIn(e) => e.to_predicate(),
            Self::Fixative(f) => f.to_predicate(),
            Self::ThermalPreservationMethod(tp) => tp.to_predicate(),
        }
    }
}

impl Default for SpecimenField {
    fn default() -> Self {
        Self::ReceivedAt
    }
}

pub type SpecimenQuery = ComplexQuery<SpecimenPredicate, SpecimenField>;

pub type SimpleSpecimenQuery = SimpleQuery<SpecimenField>;
