#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String"))]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Text))]
pub struct NonEmptyString(String);

impl std::fmt::Debug for NonEmptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Debug;

        <String as Debug>::fmt(&self.0, f)
    }
}

impl NonEmptyString {
    #[must_use]
    pub fn new(s: impl AsRef<str>) -> Option<Self> {
        let s = s.as_ref();
        if s.is_empty() {
            return None;
        }

        Some(Self(s.to_owned()))
    }
}

impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> Self {
        value.0
    }
}

impl PartialEq<str> for NonEmptyString {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for NonEmptyString {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl std::fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Display;

        <String as Display>::fmt(&self.0, f)
    }
}

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
#[error("string cannot be empty")]
pub struct Error;

impl TryFrom<String> for NonEmptyString {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(Error)
    }
}

#[cfg(feature = "diesel")]
mod diesel_impls {
    use cellnoor_schema::sql_types::CaseInsensitiveText;
    use diesel::{
        deserialize::FromSql,
        pg::{Pg, PgValue},
        serialize::{Output, ToSql},
        sql_types::{Nullable, Text},
    };

    use super::NonEmptyString;

    impl FromSql<Text, Pg> for NonEmptyString {
        fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {
            <String as FromSql<Text, Pg>>::from_sql(bytes).map(Self)
        }
    }

    impl ToSql<Text, Pg> for NonEmptyString {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
            <String as ToSql<Text, Pg>>::to_sql(&self.0, out)
        }
    }

    impl ToSql<Nullable<CaseInsensitiveText>, Pg> for NonEmptyString {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
            <Self as ToSql<Nullable<Text>, Pg>>::to_sql(self, out)
        }
    }
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod serde_tests {
    use pretty_assertions::assert_eq;

    use crate::NonEmptyString;

    #[rstest::rstest]
    fn deserialize_empty_string_fails() {
        let result: Result<Vec<NonEmptyString>, _> = serde_json::from_str(r#"[""]"#);

        assert!(result.is_err())
    }

    #[rstest::rstest]
    fn deserialize_non_empty_string_succeeds() {
        let deserialized: [NonEmptyString; 1] = serde_json::from_str(r#"["string"]"#).unwrap();

        assert_eq!(deserialized, ["string"]);
    }
}

#[cfg(feature = "diesel")]
#[cfg(test)]
mod diesel_tests {
    use diesel::{
        serialize::{Output, ToSql},
        sql_query,
        sql_types::Text,
        sqlite::Sqlite,
    };
    use pretty_assertions::assert_eq;

    use super::NonEmptyString;

    impl ToSql<Text, diesel::sqlite::Sqlite> for NonEmptyString {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> diesel::serialize::Result {
            <String as ToSql<Text, Sqlite>>::to_sql(&self.0, out)
        }
    }

    #[rstest::rstest]
    fn is_diesel_compatible() {
        use diesel::{RunQueryDsl, prelude::*};

        diesel::table! {
            table_with_strings(id) {
                id -> Integer,
                string -> Text,
                optional_string -> Nullable<Text>
            }
        }

        #[derive(Insertable)]
        struct TableWithString {
            string: NonEmptyString,
            optional_string: Option<NonEmptyString>,
        }

        let mut conn = diesel::SqliteConnection::establish(":memory:").unwrap();

        sql_query("create table table_with_strings (string text not null, optional_string text);")
            .execute(&mut conn)
            .unwrap();

        let n = diesel::insert_into(table_with_strings::table)
            .values(TableWithString {
                string: NonEmptyString::new("string").unwrap(),
                optional_string: NonEmptyString::new("string"),
            })
            .execute(&mut conn)
            .unwrap();

        assert_eq!(n, 1);
    }
}
