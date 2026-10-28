use macro_attributes::base_model;

use crate::{nonempty::NonemptyVec, query::OrderField};

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

impl<T: OrderField> OrderBySet<T> {
    /// The fields to order by, in order.
    pub fn iter(&self) -> impl Iterator<Item = OrderBy<T>> {
        match self {
            Self::One(order_by) => std::slice::from_ref(order_by).iter(),
            Self::Many(order_bys) => order_bys.as_ref().iter(),
        }
        .copied()
    }
}
