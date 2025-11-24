use default_vec::DefaultVec;

#[derive(Clone, Debug, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Query<F, O>
where
    O: Default,
{
    pub(crate) filter: Option<F>,
    pub(crate) limit: i64,
    pub(crate) offset: i64,
    pub(crate) order_by: DefaultVec<O>,
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

    pub fn filter(&self) -> Option<&F> {
        self.filter.as_ref()
    }

    pub fn limit(&self) -> i64 {
        self.limit
    }

    pub fn offset(&self) -> i64 {
        self.offset
    }

    pub fn order_by(&self) -> &DefaultVec<O> {
        &self.order_by
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

    #[test]
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
