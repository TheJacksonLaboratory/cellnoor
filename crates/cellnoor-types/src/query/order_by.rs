use std::collections::VecDeque;

use macro_attributes::base_model;

#[base_model]
#[derive(Copy)]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", schemars(inline))]
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
#[cfg_attr(feature = "schemars", schemars(inline))]
pub enum OrderBySet<T>
where
    T: Default,
{
    One(OrderBy<T>),
    Many(VecDeque<OrderBy<T>>),
}

impl<T> Default for OrderBySet<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::One(OrderBy::default())
    }
}

impl<T> OrderBySet<T>
where
    T: Default + Copy,
{
    pub fn push_front(&mut self, value: OrderBy<T>) {
        match self {
            Self::One(original) => {
                *self = Self::Many(VecDeque::from([value, *original]));
            }
            Self::Many(originals) => originals.push_front(value),
        };
    }
}
