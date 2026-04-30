/// A recursive data-structure to store arbitrarily-combined boolean predicates.
///
/// This structure can model arbitrarily complex expressions to filter database records. The type-parameter `P`
/// represents the base case or "leaf" of the filter, so it should represent a single boolean predicate. Typically this
/// is an `enum` of possible conditions.
///
/// The following example would filter for people named "Ahmed" OR "Nicole" under the age of 30:
/// ```
/// enum PersonField {
///     Name(StringOperator),
///     Age(ScalarOperator<i32>)
/// }
///
/// type PersonFilter = Filter<PersonField>;
///
/// let age_filter = PersonField::Age(ScalarOperator::Lt(30));
/// let age_filter = Filter::Leaf(age_filter);
///
/// // Use the `From` trait to convert a `ScalarOperator` into a `StringOperator`
/// let name_filter = PersonField::Name(ScalarOperator::In(vec!["Ahmed".to_owned(), "Nicole".to_owned()]).into());
/// let name_filter = Filter::Leaf(name_filter);
///
/// let combined_filter = Filter::AllOf(vec![name_filter, age_filter]);
/// ```
///
/// This could also have been represented as:
/// ```
/// let age_filter = Filter::Leaf(PersonField::Age(ScalarOperator::Lt(30)));
///
/// let ahmed_filter = Filter::Leaf(PersonField::Name(ScalarOperator::Eq("Ahmed".to_owned()).into()));
/// let nicole_filter = Filter::Leaf(PersonField::Name(ScalarOperator::Eq("Nicole".to_owned()).into()));
/// let name_filter = Filter::AnyOf(vec![ahmed_filter, nicole_filter]);
///
/// let combined_filter = Filter::AllOf(vec![age_filter, name_filter]);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub(crate) enum Filter<P> {
    /// Combines these conditions with logical `and`
    AllOf(Vec<Filter<P>>),
    /// Combines these conditions with logical `or`
    AnyOf(Vec<Filter<P>>),
    /// Negates this condition with logical `not`
    Not(Box<Filter<P>>),
    #[cfg_attr(feature = "serde", serde(untagged))]
    /// Apply just one boolean predicate
    Leaf(P),
}

/// A comparison operator for any scalar value.
///
/// See [Filter] for an example of usage.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub(crate) enum ScalarOperator<T> {
    /// equals (`=`)
    Eq(T),
    /// less than (`<`)
    Lt(T),
    /// less than or equal to (`<=`)
    Lte(T),
    /// greater than (`>`)
    Gt(T),
    /// greater than or equal to (`>=`)
    Gte(T),
    /// is contained in (`in`)
    In(Vec<T>),
    /// equals (`=`), but (de)serializes as `{"field": "value"}` instead of `{"field": {"eq": "value"}}`
    #[cfg(feature = "serde")]
    #[serde(untagged)]
    ImplicitEq(T),
}

/// A comparison operator for string values.
///
/// This is a superset of [ScalarOperator] and adds two string-specific methods present in PostgreSQL:
/// 1. [`like`](https://www.postgresql.org/docs/current/functions-matching.html#FUNCTIONS-LIKE)
/// 2. [Trigram similar](https://www.postgresql.org/docs/current/pgtrgm.html#PGTRGM-FUNCS-OPS)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub(crate) enum StringOperator {
    /// PostgreSQL `like`
    Like(String),
    /// PostgreSQL trigram similarity
    Trgm(String),
    /// All other operators
    #[cfg_attr(feature = "serde", serde(untagged))]
    Other(ScalarOperator<String>),
}

impl From<ScalarOperator<String>> for StringOperator {
    fn from(value: ScalarOperator<String>) -> Self {
        Self::Other(value)
    }
}
