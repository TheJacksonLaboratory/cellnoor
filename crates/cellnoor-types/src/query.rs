use macro_attributes::base_model;

use crate::query::{
    filter::Filter,
    order_by::{OrderBy, OrderBySet},
};

pub(crate) mod filter;
pub(crate) mod order_by;

// Most of the time, we're ordering by a time-field, so we want to see most recent first
pub trait DefaultDesc {
    fn default_desc() -> bool {
        true
    }
}

#[base_model]
#[derive(Copy, Default)]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct SimpleQuery<O>
where
    O: Default + DefaultDesc,
{
    pub limit: Option<i64>,
    pub offset: i64,
    pub order_by_field: O,
    #[cfg_attr(feature = "serde", serde(default = "O::default_desc"))]
    pub order_by_desc: bool,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", schemars(rename = "{P}Query"))]
pub struct ComplexQuery<P, O>
where
    O: Default,
{
    pub filter: Option<Filter<P>>,
    pub limit: Option<i64>,
    pub offset: i64,
    pub order_by: OrderBySet<O>,
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
        }
    }
}

impl<P, O> ComplexQuery<P, O>
where
    O: Default + DefaultDesc,
{
    pub fn from_filter(predicate: P) -> Self {
        Self {
            filter: Some(predicate.into()),
            ..Default::default()
        }
    }

    pub fn from_simple_query(
        SimpleQuery {
            limit,
            offset,
            order_by_field,
            order_by_desc,
        }: SimpleQuery<O>,
    ) -> Self {
        Self {
            limit,
            offset,
            order_by: OrderBySet::One(OrderBy {
                field: order_by_field,
                desc: order_by_desc,
            }),
            ..Default::default()
        }
    }
}

impl<P, O> From<P> for ComplexQuery<P, O>
where
    O: Default + DefaultDesc,
{
    fn from(pred: P) -> Self {
        Self::from_filter(pred)
    }
}
