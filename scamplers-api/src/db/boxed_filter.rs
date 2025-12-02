use diesel::{BoxableExpression, pg::Pg, prelude::*, sql_types::Bool};

pub type BoxedFilter<'a, QS> = Box<dyn BoxableExpression<QS, Pg, SqlType = Bool> + 'a>;

pub trait BoxedFilterExt<'a, QS> {
    fn new() -> Self;
    fn and_condition<F>(self, other: F) -> Self
    where
        F: BoxableExpression<QS, Pg, SqlType = Bool> + 'a;
}

impl<'a, QS: 'a> BoxedFilterExt<'a, QS> for BoxedFilter<'a, QS> {
    fn new() -> Self {
        Box::new(diesel::dsl::sql::<Bool>("true"))
    }

    fn and_condition<F>(self, other: F) -> Self
    where
        F: BoxableExpression<QS, Pg, SqlType = Bool> + 'a,
    {
        let other: BoxedFilter<QS> = Box::new(other);

        Box::new(self.and(other))
    }
}

pub trait ToBoxedFilter<'a, QS> {
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS>;
}

impl<'a, QS: 'a, T> ToBoxedFilter<'a, QS> for Option<T>
where
    T: ToBoxedFilter<'a, QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Some(filter) = self else {
            return BoxedFilter::new();
        };

        filter.to_boxed_filter()
    }
}
