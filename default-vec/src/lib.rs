#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct DefaultVec<T>(Vec<T>);

impl<T> DefaultVec<T> {
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, value: T) {
        self.0.push(value);
    }
}

impl<T> Default for DefaultVec<T>
where
    T: Default,
{
    fn default() -> Self {
        Self(vec![T::default()])
    }
}

#[allow(clippy::into_iter_without_iter)]
impl<'a, T> IntoIterator for &'a DefaultVec<T> {
    type IntoIter = <&'a Vec<T> as IntoIterator>::IntoIter;
    type Item = <&'a Vec<T> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T> AsMut<[T]> for DefaultVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<T> AsRef<[T]> for DefaultVec<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T> From<T> for DefaultVec<T> {
    fn from(value: T) -> Self {
        Self(vec![value])
    }
}

impl<T, const N: usize> From<[T; N]> for DefaultVec<T> {
    fn from(value: [T; N]) -> Self {
        Self(value.into())
    }
}

#[cfg(feature = "diesel")]
mod diesel_impl {
    use diesel::{AppearsOnTable, Expression, pg::Pg, query_builder::QueryFragment};

    use super::DefaultVec;

    impl<T> Expression for DefaultVec<T>
    where
        Vec<T>: Expression,
    {
        type SqlType = <Vec<T> as Expression>::SqlType;
    }

    impl<T, U> AppearsOnTable<U> for DefaultVec<T>
    where
        T: AppearsOnTable<U>,
        Vec<T>: Expression,
    {
    }

    impl<T> QueryFragment<Pg> for DefaultVec<T>
    where
        T: QueryFragment<Pg> + PartialEq,
    {
        fn walk_ast<'b>(
            &'b self,
            mut pass: diesel::query_builder::AstPass<'_, 'b, Pg>,
        ) -> diesel::QueryResult<()> {
            for item in self {
                item.walk_ast(pass.reborrow())?;
                if item != self.into_iter().last().unwrap() {
                    pass.push_sql(",");
                }
            }

            Ok(())
        }
    }
}
