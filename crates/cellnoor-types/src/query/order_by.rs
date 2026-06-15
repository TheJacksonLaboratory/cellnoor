use macro_attributes::base_model;
use nonempty::NonemptyVec;

#[base_model]
#[derive(Copy)]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", schemars(rename = "OrderBy{T}"))]
pub struct OrderBy<T>
where
    T: Default,
{
    pub field: T,
    pub desc: bool,
}

impl<T> Default for OrderBy<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            field: T::default(),
            desc: true,
        }
    }
}

#[base_model]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "schemars", schemars(rename = "OrderBy{T}Set"))]
pub enum OrderBySet<T>
where
    T: Default,
{
    One(OrderBy<T>),
    Many(NonemptyVec<OrderBy<T>>),
}

impl<T> Default for OrderBySet<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::One(OrderBy::default())
    }
}
