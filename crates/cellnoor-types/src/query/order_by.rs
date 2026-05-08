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
    Many(Vec<OrderBy<T>>),
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
    T: Default + Copy + AsRef<str>,
{
    pub fn to_order_by_clause(&self) -> String {
        fn direction(desc: bool) -> &'static str {
            if desc { "desc" } else { "asc" }
        }

        match self {
            Self::One(OrderBy { field, desc }) => {
                format!("order by {} {}", field.as_ref(), direction(*desc))
            }
            Self::Many(fields) => {
                if fields.is_empty() {
                    return String::new();
                }

                let mut clause = String::with_capacity(fields.len() * 16);
                clause.push_str("order by ");

                for OrderBy { field, desc } in fields {
                    clause.push_str(field.as_ref());
                    clause.push(' ');

                    clause.push_str(direction(*desc));
                    clause.push(' ');
                }

                clause
            }
        }
    }
}
