use macro_attributes::base_model;

use crate::query::{
    filter::Filter,
    order_by::{OrderBy, OrderBySet},
};

pub(crate) mod filter;
pub(crate) mod order_by;

#[base_model]
#[derive(Copy, Default)]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct SimpleQuery<O>
where
    O: Default,
{
    pub limit: Option<i32>,
    pub offset: i32,
    pub detailed: bool,
    pub order_by_field: O,
    pub order_by_desc: bool,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct ComplexQuery<P, O>
where
    O: Default,
{
    pub filter: Option<Filter<P>>,
    pub limit: Option<i32>,
    pub offset: i32,
    pub order_by: OrderBySet<O>,
    pub detailed: bool,
}

impl<F, O> Default for ComplexQuery<F, O>
where
    O: Default,
{
    fn default() -> Self {
        Self {
            filter: None,
            limit: None,
            offset: 0,
            order_by: OrderBySet::default(),
            detailed: false,
        }
    }
}

impl<P, O> ComplexQuery<P, O>
where
    O: Default,
{
    pub fn from_filter(predicate: P, detailed: bool) -> Self {
        Self {
            filter: Some(predicate.into()),
            detailed,
            ..Default::default()
        }
    }

    pub fn from_simple_query(
        SimpleQuery {
            limit,
            offset,
            detailed,
            order_by_field,
            order_by_desc,
        }: SimpleQuery<O>,
    ) -> Self {
        Self {
            limit,
            offset,
            detailed,
            order_by: OrderBySet::One(OrderBy {
                field: order_by_field,
                desc: order_by_desc,
            }),
            ..Default::default()
        }
    }
}
