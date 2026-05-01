use macro_attributes::field_enum;
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;
use serde_json::Value;

#[cfg(feature = "postgres-types")]
use crate::query::filter::AsPredicate;
use crate::{
    query::{
        Query,
        filter::{Filter, ScalarOperator, StringOperator, TimestampOperator, UuidOperator},
        order_by::{OrderDirection, OrderingField},
    },
    specimen::{
        Fixative, Species, SpecimenType, ThermalPreservationMethod,
        creation::block::BlockEmbeddingMatrix,
    },
};

pub type SpeciesOperator = ScalarOperator<Species>;
pub type SpecimenTypeOperator = ScalarOperator<SpecimenType>;
pub type BlockEmbeddingMatrixOperator = ScalarOperator<BlockEmbeddingMatrix>;
pub type FixativeOperator = ScalarOperator<Fixative>;
pub type ThermalPreservationMethodOperator = ScalarOperator<ThermalPreservationMethod>;

#[field_enum]
#[strum(prefix = "specimen.")]
pub enum SpecimenField<U, S, T, Sp, Ty, E, F, Tp> {
    Id(U),
    ReadableId(S),
    Name(S),
    SubmittedBy(U),
    ProjectId(U),
    ReceivedAt(T),
    Species(Sp),
    HostSpecies(Sp),
    ReturnedAt(T),
    ReturnedBy(U),
    Type(Ty),
    EmbeddedIn(E),
    Fixative(F),
    ThermalPreservationMethod(Tp),
    Tissue(S),
}

pub type SpecimenPredicate = SpecimenField<
    UuidOperator,
    StringOperator,
    TimestampOperator,
    SpeciesOperator,
    SpecimenTypeOperator,
    BlockEmbeddingMatrixOperator,
    FixativeOperator,
    ThermalPreservationMethodOperator,
>;

#[cfg(feature = "postgres-types")]
impl AsPredicate for SpecimenPredicate {
    fn as_predicate(&self) -> (&'static str, &dyn ToSql) {
        match self {
            Self::Id(u) | Self::SubmittedBy(u) | Self::ProjectId(u) | Self::ReturnedBy(u) => {
                u.as_predicate()
            }
            Self::ReadableId(s) | Self::Name(s) | Self::Tissue(s) => s.as_predicate(),
            Self::ReceivedAt(t) | Self::ReturnedAt(t) => t.as_predicate(),
            Self::Species(sp) | Self::HostSpecies(sp) => sp.as_predicate(),
            Self::Type(ty) => ty.as_predicate(),
            Self::EmbeddedIn(e) => e.as_predicate(),
            Self::Fixative(f) => f.as_predicate(),
            Self::ThermalPreservationMethod(tp) => tp.as_predicate(),
        }
    }
}

pub type SpecimenFilter = Filter<SpecimenPredicate>;

pub type SpecimenOrderBy = SpecimenField<
    OrderDirection,
    OrderDirection,
    OrderDirection,
    OrderDirection,
    OrderDirection,
    OrderDirection,
    OrderDirection,
    OrderDirection,
>;

impl OrderingField for SpecimenOrderBy {
    fn direction(self) -> OrderDirection {
        match self {
            Self::Id(d)
            | Self::ReadableId(d)
            | Self::Name(d)
            | Self::SubmittedBy(d)
            | Self::ProjectId(d)
            | Self::ReceivedAt(d)
            | Self::Species(d)
            | Self::HostSpecies(d)
            | Self::ReturnedAt(d)
            | Self::ReturnedBy(d)
            | Self::Type(d)
            | Self::EmbeddedIn(d)
            | Self::Fixative(d)
            | Self::ThermalPreservationMethod(d)
            | Self::Tissue(d) => d,
        }
    }
}

impl Default for SpecimenOrderBy {
    fn default() -> Self {
        Self::ReceivedAt(OrderDirection::Desc)
    }
}

pub type SpecimenQuery = Query<SpecimenFilter, SpecimenOrderBy>;
