#![allow(clippy::doc_markdown)]

#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "{P}Filter"))]
pub enum Filter<P> {
    /// Combines these predicates with logical and
    AllOf(Vec<Filter<P>>),
    /// Combines these predicates with logical or
    AnyOf(Vec<Filter<P>>),
    /// Negates this predicate with logical not
    Not(Box<Filter<P>>),
    #[cfg_attr(feature = "serde", serde(untagged))]
    /// Apply just one boolean predicate
    Leaf(P),
}

impl<P> From<P> for Filter<P> {
    fn from(predicate: P) -> Self {
        Self::Leaf(predicate)
    }
}

/// A comparison that renders as a SQL operator and a bind value.
#[cfg(feature = "postgres-types")]
pub trait SqlOperator {
    fn as_sql_operator_and_value(&self) -> (&'static str, &(dyn ToSql + Sync));
}

/// One boolean comparison against one column of a relation.
///
/// A read selects the relation's whole row as a composite, so the column is
/// addressed as `(relation).column`.
#[cfg(feature = "postgres-types")]
#[derive(Clone, Copy, Debug)]
pub struct Predicate<'a> {
    pub relation: &'static str,
    pub column: &'a str,
    pub operator: &'static str,
    pub value: &'a (dyn ToSql + Sync),
}

/// A type that renders as one boolean comparison.
///
/// Implemented by the `predicate_enum` and `predicate_enum_wrapper` macros.
#[cfg(feature = "postgres-types")]
pub trait AsPredicate {
    fn as_predicate(&self) -> Predicate<'_>;
}

/// A comparison operator for any scalar value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "{T}Operator"))]
pub enum Operator<T> {
    /// equals (=)
    Eq(T),
    /// less than (<)
    Lt(T),
    /// less than or equal to (<=)
    Lte(T),
    /// greater than (>)
    Gt(T),
    /// greater than or equal to (>=)
    Gte(T),
    /// is contained in (= any($1))
    In(Vec<T>),
    /// equals (=), but (de)serializes as '{"field": "value"}' instead of
    /// '{"field": {"eq": "value"}}'
    #[cfg(feature = "serde")]
    #[serde(untagged)]
    ImplicitEq(T),
}

#[cfg(feature = "postgres-types")]
impl<T> SqlOperator for Operator<T>
where
    T: ToSql + Sync,
{
    fn as_sql_operator_and_value(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Eq(v) => ("=", v),
            Self::Lt(v) => ("<", v),
            Self::Lte(v) => ("<=", v),
            Self::Gt(v) => (">", v),
            Self::Gte(v) => (">=", v),
            Self::In(v) => ("= any", v),
            #[cfg(feature = "serde")]
            Self::ImplicitEq(v) => ("=", v),
        }
    }
}

pub type BoolOperator = Operator<bool>;

pub type I32Operator = Operator<i32>;

pub type I64Operator = Operator<i64>;

pub type F32Operator = Operator<f32>;

pub type SimpleStringOperator = Operator<String>;

pub type UuidOperator = Operator<Uuid>;

pub type TimestampOperator = Operator<jiff::Timestamp>;

pub type SimpleArrayOperator<T> = Operator<Vec<T>>;

pub type SimpleJsonOperator = Operator<serde_json::Value>;

/// A comparison operator for string values.
///
/// This is a superset of Operator<T> and adds string-specific methods present
/// in PostgreSQL:
/// 1. like (https://www.postgresql.org/docs/current/functions-matching.html#FUNCTIONS-LIKE)
/// 2. trigram similar (https://www.postgresql.org/docs/current/pgtrgm.html#PGTRGM-FUNCS-OPS)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum StringOperator {
    /// PostgreSQL like
    Like(String),
    /// PostgreSQL like any
    LikeAny(Vec<String>),
    /// PostgreSQL trigram similar to (%)
    Trgm(String),
    /// PostgreSQL trigram similar to any (% any)
    TrgmAny(Vec<String>),
    /// All other operators
    #[cfg_attr(feature = "serde", serde(untagged))]
    Simple(SimpleStringOperator),
}

#[cfg(feature = "postgres-types")]
impl SqlOperator for StringOperator {
    fn as_sql_operator_and_value(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Like(s) => ("like", s),
            Self::LikeAny(s) => ("like any", s),
            Self::Trgm(s) => ("%", s),
            Self::TrgmAny(s) => ("% any", s),
            Self::Simple(op) => op.as_sql_operator_and_value(),
        }
    }
}

impl From<SimpleStringOperator> for StringOperator {
    fn from(value: SimpleStringOperator) -> Self {
        Self::Simple(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "{T}ArrayOperator"))]
pub enum ArrayOperator<T> {
    /// PostgreSQL contains (@>)
    Contains(Vec<T>),
    /// PostgreSQL is contained in (<@)
    IsContainedIn(Vec<T>),
    /// PostgreSQL overlaps (&&)
    Overlaps(Vec<T>),
    /// All other operators
    #[cfg_attr(feature = "serde", serde(untagged))]
    Simple(SimpleArrayOperator<T>),
}

#[cfg(feature = "postgres-types")]
impl<T> SqlOperator for ArrayOperator<T>
where
    T: ToSql + Sync,
{
    fn as_sql_operator_and_value(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Contains(v) => ("@>", v),
            Self::IsContainedIn(v) => ("<@", v),
            Self::Overlaps(v) => ("&&", v),
            Self::Simple(op) => op.as_sql_operator_and_value(),
        }
    }
}

impl<T> From<SimpleArrayOperator<T>> for ArrayOperator<T> {
    fn from(value: SimpleArrayOperator<T>) -> Self {
        Self::Simple(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum JsonOperator {
    /// PostgreSQL contains (@>)
    Contains(serde_json::Value),
    /// PostgreSQL is contained in (<@)
    IsContainedIn(serde_json::Value),
    /// PostgreSQL has key (?)
    HasKey(String),
    /// PostgreSQL has any of keys (?|)
    HasAnyOfKeys(Vec<String>),
    /// PostgreSQL has all of keys (?&)
    HasAllOfKeys(Vec<String>),
    /// All other operators
    #[cfg_attr(feature = "serde", serde(untagged))]
    Simple(SimpleJsonOperator),
}

#[cfg(feature = "postgres-types")]
impl SqlOperator for JsonOperator {
    fn as_sql_operator_and_value(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Contains(v) => ("@>", v),
            Self::IsContainedIn(v) => ("<@", v),
            Self::HasKey(s) => ("?", s),
            Self::HasAnyOfKeys(s) => ("?|", s),
            Self::HasAllOfKeys(s) => ("?&", s),
            Self::Simple(op) => op.as_sql_operator_and_value(),
        }
    }
}

impl From<SimpleJsonOperator> for JsonOperator {
    fn from(value: SimpleJsonOperator) -> Self {
        Self::Simple(value)
    }
}
