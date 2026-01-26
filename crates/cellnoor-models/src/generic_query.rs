use default_vec::DefaultVec;

#[derive(Clone, Debug, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
#[cfg_attr(feature = "app", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Query<F, O>
where
    F: Default,
    O: Default,
{
    pub filter: F,
    pub limit: i64,
    pub offset: i64,
    pub order_by: DefaultVec<O>,
}

impl<F, O> Default for Query<F, O>
where
    F: Default,
    O: Default,
{
    fn default() -> Self {
        Query {
            filter: F::default(),
            limit: 500,
            offset: 0,
            order_by: DefaultVec::default(),
        }
    }
}

#[cfg_attr(feature = "builder", bon::bon)]
impl<F, O> Query<F, O>
where
    F: Default,
    O: Default,
{
    #[cfg(feature = "builder")]
    #[builder(on(_, into))]
    pub fn new(
        #[builder(field = DefaultVec::new())] order_by: DefaultVec<O>,
        #[builder(default)] filter: F,
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
            filter,
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
                filter: Filter,
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
