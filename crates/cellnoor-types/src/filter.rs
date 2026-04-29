#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
enum Filter<P> {
    AllOf(Vec<Filter<P>>),
    AnyOf(Vec<Filter<P>>),
    Not(Box<Filter<P>>),
    #[cfg_attr(feature = "serde", serde(untagged))]
    Leaf(P),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
enum UniversalOperator<V> {
    Eq(V),
    Lt(V),
    Lte(V),
    Gt(V),
    Gte(V),
    In(Vec<V>),
    #[cfg_attr(feature = "serde", serde(untagged))]
    ImplicitEq(V),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
enum StringOperator {
    Like(String),
    TrgmSimilar(String),
    #[cfg_attr(feature = "serde", serde(untagged))]
    Universal(UniversalOperator<String>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
enum Direction {
    Asc,
    Desc,
}
