/// An enum representing sorting-direction, corresponding to
/// [PostgreSQL's usage](https://www.postgresql.org/docs/current/pgtrgm.html#PGTRGM-FUNCS-OPS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
enum Direction {
    /// PostgreSQL `asc`
    Asc,
    /// PostgreSQL `desc`
    Desc,
}
