use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

#[cfg(feature = "postgres-types")]
use bytes::BytesMut;
#[cfg(feature = "postgres-types")]
use postgres_types::{ToSql, to_sql_checked};

use crate::Error;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(try_from = "Vec<T>"))]
#[cfg_attr(feature = "schemars", schemars(with = "Vec<T>"))]
pub struct NonemptyBoundedVec<T, const N: usize>(Vec<T>);

impl<T, const N: usize> Index<usize> for NonemptyBoundedVec<T, N> {
    type Output = <Vec<T> as Index<usize>>::Output;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T, const N: usize> IndexMut<usize> for NonemptyBoundedVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T, const N: usize> From<T> for NonemptyBoundedVec<T, N> {
    fn from(value: T) -> Self {
        Self(vec![value])
    }
}

impl<T, const N: usize> NonemptyBoundedVec<T, N> {
    pub fn new(v: Vec<T>) -> Result<Self, Error<Vec<T>>> {
        if v.is_empty() {
            return Err(Error(v));
        }

        if v.len() > N {
            return Err(Error(v));
        }

        Ok(Self(v))
    }
}

impl<T, const N: usize> NonemptyBoundedVec<T, N> {
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}

impl<T, const N: usize> IntoIterator for NonemptyBoundedVec<T, N> {
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a NonemptyBoundedVec<T, N> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T, const N: usize> AsRef<[T]> for NonemptyBoundedVec<T, N> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T, const N: usize> AsMut<[T]> for NonemptyBoundedVec<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<T, const N: usize> From<NonemptyBoundedVec<T, N>> for Vec<T> {
    fn from(value: NonemptyBoundedVec<T, N>) -> Self {
        value.0
    }
}

impl<T, const N: usize> TryFrom<Vec<T>> for NonemptyBoundedVec<T, N> {
    type Error = Error<Vec<T>>;

    fn try_from(value: Vec<T>) -> Result<Self, Error<Vec<T>>> {
        Self::new(value)
    }
}

#[cfg(feature = "postgres-types")]
impl<T, const N: usize> ToSql for NonemptyBoundedVec<T, N>
where
    T: ToSql,
{
    to_sql_checked!();

    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        self.0.to_sql(ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        Vec::<T>::accepts(ty)
    }
}

pub type NonemptyVec<T> = NonemptyBoundedVec<T, { usize::MAX }>;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::NonemptyBoundedVec;

    #[test]
    fn empty_vec() {
        let err = NonemptyBoundedVec::<bool, 1>::new(vec![]).unwrap_err();

        assert_eq!(err.0, Vec::<bool>::new());
    }

    #[test]
    fn long_vec() {
        let err = NonemptyBoundedVec::<_, 1>::new(vec![false, false]).unwrap_err();

        assert_eq!(err.0, vec![false, false]);
    }

    #[test]
    fn good_vec() {
        NonemptyBoundedVec::<_, 2>::new(vec![true]).unwrap();
    }
}
