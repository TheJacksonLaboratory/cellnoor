#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "Vec<T>"))]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "diesel", derive(diesel::deserialize::FromSqlRow))]
pub struct NonEmptyVec<T, const N: usize = { usize::MAX }>(Vec<T>);

impl<T, const N: usize> From<T> for NonEmptyVec<T, N> {
    fn from(value: T) -> Self {
        Self(vec![value])
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("array must have between 1 and {N} elements")]
pub struct Error<T, const N: usize>(pub Vec<T>);

impl<T, const N: usize> NonEmptyVec<T, N> {
    #[must_use]
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

impl<T, const N: usize> IntoIterator for NonEmptyVec<T, N> {
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, const N: usize> AsRef<[T]> for NonEmptyVec<T, N> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T, const N: usize> From<NonEmptyVec<T, N>> for Vec<T> {
    fn from(value: NonEmptyVec<T, N>) -> Self {
        value.0
    }
}

impl<T, const N: usize> TryFrom<Vec<T>> for NonEmptyVec<T, N> {
    type Error = Error<T, N>;

    fn try_from(value: Vec<T>) -> Result<Self, Error<T, N>> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::NonEmptyVec;

    #[test]
    fn empty_vec() {
        let err = NonEmptyVec::<bool, 1>::new(vec![]).unwrap_err();

        assert_eq!(err, super::Error(vec![]));
    }

    #[test]
    fn long_vec() {
        let err = NonEmptyVec::<_, 1>::new(vec![false, false]).unwrap_err();

        assert_eq!(err, super::Error(vec![false, false]));
    }

    #[test]
    fn good_vec() {
        NonEmptyVec::<_, 2>::new(vec![true]).unwrap();
    }
}
