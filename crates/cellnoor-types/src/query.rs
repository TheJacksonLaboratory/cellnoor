use macro_attributes::base_model;

use crate::query::order_by::OrderBy;

pub mod filter;
pub mod order_by;

#[base_model]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct Query<F, O>
where
    O: Default,
{
    pub filter: Option<F>,
    pub limit: Option<i32>,
    pub offset: i32,
    pub order_by: OrderBy<O>,
    pub detailed: bool,
}

impl<F, O> Default for Query<F, O>
where
    O: Default,
{
    fn default() -> Self {
        Self {
            filter: None,
            limit: None,
            offset: 0,
            order_by: OrderBy::default(),
            detailed: false,
        }
    }
}
