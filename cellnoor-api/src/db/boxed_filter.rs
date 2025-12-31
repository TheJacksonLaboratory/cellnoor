use diesel::{BoxableExpression, pg::Pg, prelude::*, sql_types::Bool};

pub type BoxedFilter<'a, QS> = Box<dyn BoxableExpression<QS, Pg, SqlType = Bool> + 'a>;

pub trait BoxedFilterExt<'a, QS> {
    fn new_true() -> Self;
    fn new_false() -> Self;
    fn and_condition<F>(self, other: F) -> Self
    where
        F: BoxableExpression<QS, Pg, SqlType = Bool> + 'a;
    fn or_condition<F>(self, other: F) -> Self
    where
        F: BoxableExpression<QS, Pg, SqlType = Bool> + 'a;
}

impl<'a, QS: 'a> BoxedFilterExt<'a, QS> for BoxedFilter<'a, QS> {
    fn new_true() -> Self {
        Box::new(diesel::dsl::sql::<Bool>("true"))
    }

    fn new_false() -> Self {
        Box::new(diesel::dsl::sql::<Bool>("false"))
    }

    fn and_condition<F>(self, other: F) -> Self
    where
        F: BoxableExpression<QS, Pg, SqlType = Bool> + 'a,
    {
        let other: BoxedFilter<QS> = Box::new(other);

        Box::new(self.and(other))
    }

    fn or_condition<F>(self, other: F) -> Self
    where
        F: BoxableExpression<QS, Pg, SqlType = Bool> + 'a,
    {
        let other: BoxedFilter<QS> = Box::new(other);

        Box::new(self.or(other))
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
            return BoxedFilter::new_true();
        };

        filter.to_boxed_filter()
    }
}
