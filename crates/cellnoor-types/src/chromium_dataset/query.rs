use macro_attributes::{base_model, predicate_enum, predicate_enum_wrapper, sort_field_enum};

use crate::{
    library::LibraryPredicate,
    operator::{StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, DefaultDesc, SimpleQuery, filter::Filter},
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

#[predicate_enum_wrapper]
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

impl DefaultDesc for ChromiumDatasetField {}

pub type ChromiumDatasetQuery = ComplexQuery<ChromiumDatasetPredicate, ChromiumDatasetField>;

pub type SimpleChromiumDatasetQuery = SimpleQuery<ChromiumDatasetField>;

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    #[test]
    fn filter_deserialization() {
        use crate::{
            chromium_dataset::{
                ChromiumDatasetPredicate, ChromiumDatasetPredicateInner, ChromiumDatasetQuery,
            },
            operator::SimpleStringOperator,
            order_by::{OrderBy, OrderBySet},
        };

        let untyped_filter = serde_json::json!({ "name": "some name" });
        let filter: ChromiumDatasetPredicate =
            serde_json::from_value(untyped_filter.clone()).unwrap();

        pretty_assertions::assert_eq!(
            filter,
            ChromiumDatasetPredicate::ChromiumDataset(ChromiumDatasetPredicateInner::Name(
                SimpleStringOperator::ImplicitEq("some name".to_owned()).into()
            ))
        );

        let query = serde_json::json!({"filter": untyped_filter});
        let query: ChromiumDatasetQuery = serde_json::from_value(query).unwrap();
        pretty_assertions::assert_eq!(
            query,
            ChromiumDatasetQuery {
                filter: Some(filter.into()),
                limit: None,
                offset: 0,
                order_by: OrderBySet::One(OrderBy {
                    field: crate::chromium_dataset::ChromiumDatasetField::DeliveredAt,
                    desc: true
                })
            }
        );
    }
}
