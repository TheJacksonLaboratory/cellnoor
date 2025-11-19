use std::{fmt::Display, str::FromStr};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Text))]

pub struct NonEmptyString(String);

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[error("empty string '{0}' not allowed")]
pub struct EmptyStringError(String);

impl NonEmptyString {
    pub fn new(s: String) -> Result<Self, EmptyStringError> {
        if s.is_empty() {
            return Err(EmptyStringError(s));
        }

        Ok(Self(s))
    }

    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl FromStr for NonEmptyString {
    type Err = EmptyStringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_owned())
    }
}

impl Display for NonEmptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for NonEmptyString {
    fn from(value: &str) -> Self {
        Self::from_str(value).unwrap()
    }
}

impl From<String> for NonEmptyString {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// This is essentially taken from https://github.com/MidasLamb/non-empty-string
#[cfg(feature = "serde")]
mod serde_impls {
    use std::fmt;

    use serde::{
        Serialize,
        de::{self, Deserialize, Deserializer, Unexpected, Visitor},
    };

    use super::NonEmptyString;

    impl Serialize for NonEmptyString {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.serialize(serializer)
        }
    }

    struct NonEmptyStringVisitor;

    impl<'de> Deserialize<'de> for NonEmptyString {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_string(NonEmptyStringVisitor)
        }
    }

    impl Visitor<'_> for NonEmptyStringVisitor {
        type Value = NonEmptyString;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string with length > 0")
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            NonEmptyString::new(value)
                .map_err(|e| de::Error::invalid_value(Unexpected::Str(&e.0), &self))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_string(value.to_owned())
        }
    }
}

#[cfg(feature = "diesel")]
mod diesel_impls {
    use diesel::{
        deserialize::FromSql,
        pg::{Pg, PgValue},
        serialize::{Output, ToSql},
        sql_types::Text,
    };

    use crate::NonEmptyString;

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
}

#[cfg(feature = "diesel")]
#[cfg(test)]
mod test {
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

    #[test]
    fn diesel_compatible() {
        use std::str::FromStr;

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
                string: NonEmptyString::from_str("string").unwrap(),
                optional_string: Some(NonEmptyString::from_str("string").unwrap()),
            })
            .execute(&mut conn)
            .unwrap();

        assert_eq!(n, 1);
    }
}
