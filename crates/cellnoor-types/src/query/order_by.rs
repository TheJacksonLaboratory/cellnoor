use macro_attributes::base_model;
use nonempty::NonemptyVec;

use crate::query::OrderField;

#[base_model]
#[derive(Copy)]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", schemars(rename = "OrderBy{T}"))]
pub struct OrderBy<T: OrderField> {
    pub field: T,
    pub desc: bool,
}

impl<T: OrderField> Default for OrderBy<T> {
    fn default() -> Self {
        Self {
            field: T::default_field(),
            desc: T::default_desc(),
        }
    }
}

#[base_model]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "schemars", schemars(rename = "OrderBy{T}Set"))]
pub enum OrderBySet<T>
where
    T: OrderField,
{
    One(OrderBy<T>),
    Many(NonemptyVec<OrderBy<T>>),
}

impl<T: OrderField> Default for OrderBySet<T> {
    fn default() -> Self {
        Self::One(OrderBy::default())
    }
}
