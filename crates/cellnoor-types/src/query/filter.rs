#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "{P}Filter"))]
pub enum Filter<P> {
    /// Combines these predicates with logical `and`
    AllOf(Vec<Filter<P>>),
    /// Combines these predicates with logical `or`
    AnyOf(Vec<Filter<P>>),
    /// Negates this predicate with logical `not`
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

#[cfg(feature = "postgres-types")]
pub trait ToPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync));
}

#[cfg(feature = "postgres-types")]
impl<P> Filter<P>
where
    P: AsRef<str> + ToPredicate,
{
    fn as_where_clause_inner<'a>(
        &'a self,
        bind_params: &mut Vec<&'a (dyn ToSql + Sync)>,
    ) -> String {
        match self {
            Self::Leaf(pred) => {
                let (operator, bind_param) = pred.to_predicate();

                bind_params.push(bind_param);
                // This works because Postgres's indexing for bind parameters starts at 1
                let query = format!("{} {} (${})", pred.as_ref(), operator, bind_params.len());

                query
            }
            Self::AllOf(filters) | Self::AnyOf(filters) => {
                let mut query = "".to_owned();

                let (combinator, default) = if matches!(self, Self::AllOf(_)) {
                    (" and ", "true")
                } else {
                    (" or ", "false")
                };

                if filters.is_empty() {
                    return default.to_owned();
                }

                for (i, f) in filters.iter().enumerate() {
                    let subquery = f.as_where_clause_inner(bind_params);
                    query.push_str(&format!("({subquery})"));

                    if i != filters.len() - 1 {
                        query.push_str(combinator);
                    }
                }

                query
            }
            Self::Not(filter) => {
                let query = format!("not ({})", filter.as_where_clause_inner(bind_params));

                query
            }
        }
    }

    pub(crate) fn to_where_clause(&self) -> (String, Vec<&(dyn ToSql + Sync)>) {
        // 64 is arbitrary but it's not a lot and definitely more than anyone will be
        // constructing
        let mut bind_params = Vec::with_capacity(64);

        let query = self.as_where_clause_inner(&mut bind_params);

        (format!("where {query}"), bind_params)
    }

    pub fn and(mut self, predicate: P) -> Self {
        match &mut self {
            Self::AllOf(filters) => {
                filters.push(predicate.into());
                self
            }
            Self::AnyOf(_) | Self::Not(_) | Self::Leaf(_) => {
                Self::AllOf(vec![self, predicate.into()])
            }
        }
    }
}

/// A comparison operator for any scalar value.
///
/// See [Filter] for an example of usage.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub enum ScalarOperator<T> {
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
    /// is contained in (`= any($1)`)
    In(Vec<T>),
    /// equals (`=`), but (de)serializes as `{"field": "value"}` instead of
    /// `{"field": {"eq": "value"}}`
    #[cfg(feature = "serde")]
    #[serde(untagged)]
    ImplicitEq(T),
}

#[cfg(feature = "postgres-types")]
impl<T> ToPredicate for ScalarOperator<T>
where
    T: ToSql + Sync,
{
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
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

pub type BoolOperator = ScalarOperator<bool>;

pub type I32Operator = ScalarOperator<i32>;

pub type I64Operator = ScalarOperator<i64>;

pub type SimpleStringOperator = ScalarOperator<String>;

pub type UuidOperator = ScalarOperator<Uuid>;

pub type TimestampOperator = ScalarOperator<jiff::Timestamp>;

/// A comparison operator for string values.
///
/// This is a superset of [ScalarOperator] and adds two string-specific methods
/// present in PostgreSQL:
/// 1. [`like`](https://www.postgresql.org/docs/current/functions-matching.html#FUNCTIONS-LIKE)
/// 2. [Trigram similar](https://www.postgresql.org/docs/current/pgtrgm.html#PGTRGM-FUNCS-OPS)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub enum StringOperator {
    /// PostgreSQL `like`
    Like(String),
    /// PostgreSQL trigram similarity
    Trgm(String),
    /// All other operators
    #[cfg_attr(feature = "serde", serde(untagged))]
    Simple(ScalarOperator<String>),
}

#[cfg(feature = "postgres-types")]
impl ToPredicate for StringOperator {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Like(s) => ("like", s),
            Self::Trgm(s) => ("%", s),
            Self::Simple(op) => op.to_predicate(),
        }
    }
}

impl From<ScalarOperator<String>> for StringOperator {
    fn from(value: ScalarOperator<String>) -> Self {
        Self::Simple(value)
    }
}
