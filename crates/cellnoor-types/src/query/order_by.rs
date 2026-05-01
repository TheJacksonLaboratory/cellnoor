use std::fmt::Display;

use macro_attributes::base_model;

/// An enum representing sorting-direction, corresponding to
/// [PostgreSQL's usage](https://www.postgresql.org/docs/current/pgtrgm.html#PGTRGM-FUNCS-OPS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::IntoStaticStr, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", schemars(inline))]
#[strum(serialize_all = "snake_case")]
pub enum OrderDirection {
    /// PostgreSQL `asc`
    Asc,
    /// PostgreSQL `desc`
    Desc,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub enum OrderBy<T> {
    OneField(T),
    ManyFields(Vec<T>),
}

impl<T> Default for OrderBy<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::OneField(T::default())
    }
}

pub trait OrderingField {
    fn direction(self) -> OrderDirection;
}

impl<T> OrderBy<T>
where
    T: Copy + Into<&'static str> + OrderingField,
{
    pub fn to_order_by_clause(&self) -> String {
        match self {
            Self::OneField(field) => {
                let s: &str = (*field).into();
                format!("{} {}", s, field.direction())
            }
            Self::ManyFields(fields) => {
                let mut clause = String::with_capacity(fields.len() * 16);
                for f in fields {
                    clause.push_str((*f).into());
                    clause.push(' ');
                    clause.push_str(f.direction().into());
                    clause.push(' ');
                }

                clause
            }
        }
    }
}
