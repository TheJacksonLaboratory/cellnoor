use macro_attributes::{predicate_enum, predicate_enum_wrapper, sort_field_enum};

use crate::{
    Relation,
    operator::{JsonOperator, StringOperator, TimestampOperator, UuidOperator},
    project::ProjectPredicate,
    query::{ComplexQuery, Field, OrderField, SimpleQuery, filter::Operator},
    specimen::{
        Fixative, SavedSpecimenRecord, Species, SpecimenType, ThermalPreservationMethod,
        creation::block::BlockEmbeddingMatrix,
    },
};

pub type SpeciesOperator = Operator<Species>;
pub type SpecimenTypeOperator = Operator<SpecimenType>;
pub type BlockEmbeddingMatrixOperator = Operator<BlockEmbeddingMatrix>;
pub type FixativeOperator = Operator<Fixative>;
pub type ThermalPreservationMethodOperator = Operator<ThermalPreservationMethod>;

#[predicate_enum(SpecimenField)]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub enum SpecimenPredicateInner {
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
    AdditionalData(JsonOperator),
}

#[predicate_enum_wrapper]
pub enum SpecimenPredicate {
    #[strum(transparent)]
    Project(ProjectPredicate),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(transparent)]
    Specimen(SpecimenPredicateInner),
}

impl Field for SpecimenField {
    const RELATION: &'static str = <SavedSpecimenRecord as Relation>::NAME;
}

impl OrderField for SpecimenField {
    fn default_field() -> Self {
        Self::ReceivedAt
    }
}

pub type SpecimenQuery = ComplexQuery<SpecimenPredicate, SpecimenField>;

pub type SimpleSpecimenQuery = SimpleQuery<SpecimenField>;
