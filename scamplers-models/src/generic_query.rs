#[cfg(not(feature = "typescript"))]
mod rust {
    use default_vec::DefaultVec;
    use macro_attributes::{base_model, base_model_default};

    #[base_model_default]
    #[cfg_attr(feature = "builder", derive(bon::Builder))]
    #[cfg_attr(feature = "builder", builder(on(_, into)))]
    pub struct OrderBy<O>
    where
        O: Default,
    {
        field: O,
        #[cfg_attr(feature = "builder", builder(default))]
        descending: bool,
    }

    impl<O> OrderBy<O>
    where
        O: Copy + Default,
    {
        pub fn field(&self) -> O {
            self.field
        }

        pub fn descending(&self) -> bool {
            self.descending
        }
    }

    #[base_model]
    #[serde(default)]
    pub struct Query<F, O>
    where
        O: Default,
    {
        #[serde(flatten)]
        filter: Option<F>,
        limit: i64,
        offset: i64,
        order_by: DefaultVec<OrderBy<O>>,
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

    impl<F, O> Query<F, O>
    where
        O: Default,
    {
        pub fn filter(&self) -> Option<&F> {
            self.filter.as_ref()
        }

        pub fn limit(&self) -> i64 {
            self.limit
        }

        pub fn offset(&self) -> i64 {
            self.offset
        }

        pub fn order_by(&self) -> &[OrderBy<O>] {
            self.order_by.as_ref()
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
            #[builder(field)] order_by: DefaultVec<OrderBy<O>>,
            filter: Option<F>,
            #[builder(default)] limit: i64,
            #[builder(default)] offset: i64,
        ) -> Self {
            Self {
                filter,
                limit,
                offset,
                order_by,
            }
        }

        pub fn from_filter(filter: F) -> Self {
            Self {
                filter: Some(filter),
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
        pub fn order_by(mut self, field: O, descending: bool) -> Self {
            self.order_by.push(OrderBy { field, descending });

            self
        }

        pub fn order_by_ascending(self, field: O) -> Self {
            self.order_by(field, false)
        }

        pub fn order_by_descending(self, field: O) -> Self {
            self.order_by(field, true)
        }
    }
}

#[cfg(not(feature = "typescript"))]
pub(crate) use rust::Query;

#[cfg(feature = "typescript")]
mod typescript {
    use macro_attributes::base_model;

    #[base_model]
    #[ts(optional_fields)]
    pub struct OrderBy<O>
    where
        O: ts_rs::TS,
        <O as ts_rs::TS>::OptionInnerType: ts_rs::TS,
    {
        field: O,
        descending: Option<bool>,
    }

    #[base_model]
    #[ts(optional_fields)]
    pub struct Query<F, O>
    where
        F: ts_rs::TS,
        O: ts_rs::TS,
        <O as ts_rs::TS>::OptionInnerType: ts_rs::TS,
    {
        #[serde(flatten)]
        filter: Option<F>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[ts(inline)]
        order_by: Option<Vec<OrderBy<O>>>,
    }
}

#[cfg(feature = "typescript")]
pub use typescript::Query;
