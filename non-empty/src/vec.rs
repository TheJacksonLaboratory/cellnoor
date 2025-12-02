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

impl<T, const N: usize> NonEmptyVec<T, N> {
    #[must_use]
    pub fn new(v: Vec<T>) -> Option<Self> {
        if v.is_empty() {
            return None;
        }

        if v.len() > N {
            return None;
        }

        Some(Self(v))
    }
}

impl<T, const N: usize> IntoIterator for NonEmptyVec<T, N> {
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, const N: usize> FromIterator<T> for NonEmptyVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
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

#[derive(Debug, thiserror::Error)]
#[error("array cannot be empty")]
pub struct Error;

impl<T, const N: usize> TryFrom<Vec<T>> for NonEmptyVec<T, N> {
    type Error = Error;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(Error)
    }
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod serde_tests {
    use pretty_assertions::assert_eq;

    use super::NonEmptyVec;

    #[test]
    fn deserialize_empty_array_fails() {
        let result: Result<NonEmptyVec<bool>, _> = serde_json::from_str(r#"[]"#);

        assert!(result.is_err())
    }

    #[test]
    fn deserialize_non_empty_vec_succeeds() {
        let deserialized: NonEmptyVec<bool> = serde_json::from_str(r#"[true]"#).unwrap();

        assert_eq!(deserialized, NonEmptyVec::new(vec![true]).unwrap());
    }
}
