use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    Relation,
    operator::{JsonOperator, StringOperator, TimestampOperator, UuidOperator},
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
pub enum SpecimenPredicate {
    Id(UuidOperator),
    ReadableId(StringOperator),
    Name(StringOperator),
    SubmittedBy(UuidOperator),
    ProjectId(UuidOperator),
    ReceivedOrCreatedAt(TimestampOperator),
    ParentSpecimenId(UuidOperator),
    #[strum(serialize = "species.species")]
    #[strum_discriminants(strum(serialize = "species.species"))]
    Species(SpeciesOperator),
    #[strum(serialize = "species.host_species")]
    #[strum_discriminants(strum(serialize = "species.host_species"))]
    HostSpecies(SpeciesOperator),
    GivenAt(TimestampOperator),
    GivenBy(UuidOperator),
    GivenTo(UuidOperator),
    Type(SpecimenTypeOperator),
    EmbeddedIn(BlockEmbeddingMatrixOperator),
    Fixative(FixativeOperator),
    ThermalPreservationMethod(ThermalPreservationMethodOperator),
    Tissue(StringOperator),
    AdditionalData(JsonOperator),
}

impl Field for SpecimenField {
    const RELATION: &'static str = <SavedSpecimenRecord as Relation>::NAME;
}

impl OrderField for SpecimenField {
    fn default_field() -> Self {
        Self::ReceivedOrCreatedAt
    }
}

pub type SpecimenQuery = ComplexQuery<SpecimenPredicate, SpecimenField>;

pub type SimpleSpecimenQuery = SimpleQuery<SpecimenField>;
