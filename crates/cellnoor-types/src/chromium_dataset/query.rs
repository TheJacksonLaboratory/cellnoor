use macro_attributes::{base_model, predicate_enum, sort_field_enum};

use crate::{
    library::LibraryPredicate,
    operator::{StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, SimpleQuery, filter::Filter},
    specimen::SpecimenPredicate,
    tenx_assay::TenxAssayPredicate,
};

#[predicate_enum]
#[strum(prefix = "(chromium_dataset).")]
#[strum_discriminants(
    name(ChromiumDatasetField),
    sort_field_enum,
    strum(prefix = "(chromium_dataset).")
)]
pub enum ChromiumDatasetPredicateInner {
    Id(UuidOperator),
    Name(StringOperator),
    DeliveredAt(TimestampOperator),
}

#[base_model]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(strum::IntoStaticStr)]
pub enum ChromiumDatasetPredicate {
    #[strum(transparent)]
    Specimen(SpecimenPredicate),
    #[strum(transparent)]
    TenxAssay(TenxAssayPredicate),
    #[strum(transparent)]
    Library(LibraryPredicate),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(transparent)]
    ChromiumDataset(ChromiumDatasetPredicateInner),
}

impl ChromiumDatasetPredicate {
    pub fn field_name(&self) -> &'static str {
        match self {
            Self::Specimen(p) => p.field_name(),
            Self::TenxAssay(p) => p.field_name(),
            Self::Library(p) => p.field_name(),
            Self::ChromiumDataset(p) => p.field_name(),
        }
    }
}

impl From<SpecimenPredicate> for ChromiumDatasetPredicate {
    fn from(value: SpecimenPredicate) -> Self {
        Self::Specimen(value)
    }
}

impl From<TenxAssayPredicate> for ChromiumDatasetPredicate {
    fn from(value: TenxAssayPredicate) -> Self {
        Self::TenxAssay(value)
    }
}

impl From<LibraryPredicate> for ChromiumDatasetPredicate {
    fn from(value: LibraryPredicate) -> Self {
        Self::Library(value)
    }
}

impl From<ChromiumDatasetPredicateInner> for ChromiumDatasetPredicate {
    fn from(value: ChromiumDatasetPredicateInner) -> Self {
        Self::ChromiumDataset(value)
    }
}

impl From<ChromiumDatasetPredicateInner> for Filter<ChromiumDatasetPredicate> {
    fn from(value: ChromiumDatasetPredicateInner) -> Self {
        Self::Leaf(value.into())
    }
}

impl Default for ChromiumDatasetField {
    fn default() -> Self {
        Self::DeliveredAt
    }
}

pub type ChromiumDatasetQuery = ComplexQuery<ChromiumDatasetPredicate, ChromiumDatasetField>;

pub type SimpleChromiumDatasetQuery = SimpleQuery<ChromiumDatasetField>;
