use default_vec::DefaultVec;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Query<F, O>
where
    O: Default,
{
    pub filter: Option<F>,
    pub limit: i64,
    pub offset: i64,
    pub order_by: DefaultVec<O>,
}

impl<F, O> Default for Query<F, O>
where
    O: Default,
{
    fn default() -> Self {
        Query {
            filter: None,
            limit: 500,
            offset: 0,
            order_by: DefaultVec::default(),
        }
    }
}

#[cfg_attr(feature = "builder", bon::bon)]
impl<F, O> Query<F, O>
where
    O: Default,
{
    #[cfg(feature = "builder")]
    #[builder(on(_, into))]
    pub fn new(
        #[builder(field = DefaultVec::new())] order_by: DefaultVec<O>,
        filter: Option<F>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Self {
        let default = Self::default();

        Self {
            filter,
            order_by,
            limit: limit.unwrap_or(default.limit),
            offset: offset.unwrap_or(default.offset),
        }
    }

    pub fn from_filter(filter: F) -> Self {
        Self {
            filter: Some(filter),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn default_with_no_limit() -> Self {
        Self {
            limit: i64::MAX,
            ..Default::default()
        }
    }
}

pub trait SetParentId<T>
where
    Uuid: From<T>,
{
    fn parent_ids_mut(&mut self) -> &mut Option<Vec<Uuid>>;

    fn set_parent_id(&mut self, id: T) {
        let parent_ids = self.parent_ids_mut();
        *parent_ids = Some(vec![id.into()]);
    }
}

pub trait WithParentId<T> {
    fn with_parent_id(id: T) -> Self;
}

impl<T, U> WithParentId<U> for T
where
    T: Default + SetParentId<U>,
    Uuid: From<U>,
{
    fn with_parent_id(id: U) -> Self {
        let mut filter = Self::default();
        filter.set_parent_id(id);

        filter
    }
}

impl<F, O> Query<F, O>
where
    O: Default,
{
    pub fn set_parent_id<T>(&mut self, id: T)
    where
        F: SetParentId<T> + WithParentId<T>,
        Uuid: From<T>,
    {
        if let Some(filter) = &mut self.filter {
            filter.set_parent_id(id);
        } else {
            self.filter = Some(F::with_parent_id(id));
        }
    }
}

#[cfg(feature = "builder")]
impl<F, O, S> QueryBuilder<F, O, S>
where
    F: Default,
    O: Default,
    S: query_builder::State,
{
    pub fn order_by(mut self, field: O) -> Self {
        self.order_by.push(field);

        self
    }
}

#[cfg(all(test, feature = "builder", feature = "app"))]
mod tests {
    use pretty_assertions::assert_eq;

    use super::Query;

    #[rstest::rstest]
    fn query_builder() {
        #[derive(Debug, Default, PartialEq)]
        struct Filter;

        #[derive(Debug, PartialEq)]
        enum OrderBy {
            Field1 { descending: bool },
            Field2 { descending: bool },
        }

        impl Default for OrderBy {
            fn default() -> Self {
                Self::Field1 { descending: false }
            }
        }

        let q = Query::<Filter, _>::builder()
            .order_by(OrderBy::Field1 { descending: false })
            .order_by(OrderBy::Field2 { descending: true })
            .build();

        assert_eq!(
            q,
            Query {
                filter: None,
                limit: 500,
                offset: 0,
                order_by: [
                    OrderBy::Field1 { descending: false },
                    OrderBy::Field2 { descending: true },
                ]
                .into()
            }
        )
    }
}
