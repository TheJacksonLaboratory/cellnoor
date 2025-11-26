#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "Vec<T>"))]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "diesel", derive(diesel::deserialize::FromSqlRow))]
pub struct NonEmptyVec<T>(Vec<T>);

impl<T> NonEmptyVec<T> {
    #[must_use]
    pub fn new(v: Vec<T>) -> Option<Self> {
        if v.is_empty() {
            return None;
        }

        Some(Self(v))
    }
}

impl<T> AsRef<[T]> for NonEmptyVec<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T> From<NonEmptyVec<T>> for Vec<T> {
    fn from(value: NonEmptyVec<T>) -> Self {
        value.0
    }
}

#[derive(Debug, thiserror::Error)]
#[error("string cannot be empty")]
pub struct Error;

impl<T> TryFrom<Vec<T>> for NonEmptyVec<T> {
    type Error = Error;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(Error)
    }
}

#[cfg(feature = "diesel")]
mod diesel_impls {
    use std::fmt::Debug;

    use diesel::{
        deserialize::FromSql,
        pg::{Pg, PgValue},
        serialize::{Output, ToSql},
        sql_types::{Array, SqlType},
    };

    use super::NonEmptyVec;

    impl<T, U> FromSql<Array<T>, Pg> for NonEmptyVec<U>
    where
        T: SqlType,
        Vec<U>: FromSql<Array<T>, Pg>,
    {
        fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {
            <Vec<U> as FromSql<Array<T>, Pg>>::from_sql(bytes).map(Self)
        }
    }

    impl<T, U> ToSql<Array<T>, Pg> for NonEmptyVec<U>
    where
        U: Debug,
        Vec<U>: ToSql<Array<T>, Pg>,
    {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
            <Vec<U> as ToSql<Array<T>, Pg>>::to_sql(&self.0, out)
        }
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
