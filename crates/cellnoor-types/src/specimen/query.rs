use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{JsonOperator, StringOperator, TimestampOperator, UuidOperator},
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
    AdditionalData(JsonOperator),
}

impl Default for SpecimenField {
    fn default() -> Self {
        Self::ReceivedAt
    }
}

pub type SpecimenQuery = ComplexQuery<SpecimenPredicate, SpecimenField>;

pub type SimpleSpecimenQuery = SimpleQuery<SpecimenField>;
