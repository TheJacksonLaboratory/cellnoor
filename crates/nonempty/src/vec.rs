use std::fmt::Debug;

pub use inner::NonemptyBoundedVec;

#[must_use]
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[error("array must have between 1 and {N} elements")]
pub struct Error<T, const N: usize>(pub Vec<T>);

#[cfg(feature = "postgres-types")]
mod inner {
    use postgres_types::{FromSql, ToSql};

    use super::Error;

    #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, FromSql, ToSql)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "serde", serde(try_from = "Vec<T>"))]
    #[cfg_attr(feature = "schemars", schemars(with = "Vec<T>"))]
    #[postgres(transparent)]
    pub struct NonemptyBoundedVec<T, const N: usize>(Vec<T>)
    where
        T: ToSql;

    impl<T, const N: usize> From<T> for NonemptyBoundedVec<T, N>
    where
        T: ToSql,
    {
        fn from(value: T) -> Self {
            Self(vec![value])
        }
    }

    impl<T, const N: usize> NonemptyBoundedVec<T, N>
    where
        T: ToSql,
    {
        pub fn new(v: Vec<T>) -> Result<Self, Error<T, N>> {
            if v.is_empty() {
                return Err(Error(v));
            }

            if v.len() > N {
                return Err(Error(v));
            }

            Ok(Self(v))
        }
    }

    impl<T, const N: usize> IntoIterator for NonemptyBoundedVec<T, N>
    where
        T: ToSql,
    {
        type IntoIter = <Vec<T> as IntoIterator>::IntoIter;
        type Item = T;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl<T, const N: usize> AsRef<[T]> for NonemptyBoundedVec<T, N>
    where
        T: ToSql,
    {
        fn as_ref(&self) -> &[T] {
            &self.0
        }
    }

    impl<T, const N: usize> From<NonemptyBoundedVec<T, N>> for Vec<T>
    where
        T: ToSql,
    {
        fn from(value: NonemptyBoundedVec<T, N>) -> Self {
            value.0
        }
    }

    impl<T, const N: usize> TryFrom<Vec<T>> for NonemptyBoundedVec<T, N>
    where
        T: ToSql,
    {
        type Error = Error<T, N>;

        fn try_from(value: Vec<T>) -> Result<Self, Error<T, N>> {
            Self::new(value)
        }
    }
}

#[cfg(not(feature = "postgres-types"))]
mod inner {
    use super::Error;

    #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "serde", serde(try_from = "Vec<T>"))]
    #[cfg_attr(feature = "schemars", schemars(with = "Vec<T>"))]
    pub struct NonemptyBoundedVec<T, const N: usize>(Vec<T>);

    impl<T, const N: usize> From<T> for NonemptyBoundedVec<T, N> {
        fn from(value: T) -> Self {
            Self(vec![value])
        }
    }

    impl<T, const N: usize> NonemptyBoundedVec<T, N> {
        pub fn new(v: Vec<T>) -> Result<Self, Error<T, N>> {
            if v.is_empty() {
                return Err(Error(v));
            }

            if v.len() > N {
                return Err(Error(v));
            }

            Ok(Self(v))
        }
    }

    impl<T, const N: usize> IntoIterator for NonemptyBoundedVec<T, N> {
        type IntoIter = <Vec<T> as IntoIterator>::IntoIter;
        type Item = T;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl<T, const N: usize> AsRef<[T]> for NonemptyBoundedVec<T, N> {
        fn as_ref(&self) -> &[T] {
            &self.0
        }
    }

    impl<T, const N: usize> From<NonemptyBoundedVec<T, N>> for Vec<T> {
        fn from(value: NonemptyBoundedVec<T, N>) -> Self {
            value.0
        }
    }

    impl<T, const N: usize> TryFrom<Vec<T>> for NonemptyBoundedVec<T, N> {
        type Error = Error<T, N>;

        fn try_from(value: Vec<T>) -> Result<Self, Error<T, N>> {
            Self::new(value)
        }
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

        assert_eq!(err, super::Error(vec![]));
    }

    #[test]
    fn long_vec() {
        let err = NonemptyBoundedVec::<_, 1>::new(vec![false, false]).unwrap_err();

        assert_eq!(err, super::Error(vec![false, false]));
    }

    #[test]
    fn good_vec() {
        NonemptyBoundedVec::<_, 2>::new(vec![true]).unwrap();
    }
}
