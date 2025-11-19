use std::fmt::Display;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Text))]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(s: String) -> Option<Self> {
        if s.is_empty() {
            return None;
        }

        Some(Self(s))
    }
}

impl Display for NonEmptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[macro_export]
macro_rules! non_empty_string {
    ("") => {
        compile_error!("string cannot be empty");
    };
    ($s:literal) => {
        $crate::NonEmptyString::new(format!($s)).unwrap()
    };
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
                .ok_or_else(|| de::Error::invalid_value(Unexpected::Str(""), &self))
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

#[cfg(test)]
mod test {
    #[cfg(feature = "diesel")]
    use diesel::{
        serialize::{Output, ToSql},
        sql_query,
        sql_types::Text,
        sqlite::Sqlite,
    };
    use pretty_assertions::assert_eq;

    use super::NonEmptyString;

    #[test]
    fn macro_() {
        non_empty_string!("string");
    }

    #[cfg(feature = "diesel")]
    impl ToSql<Text, diesel::sqlite::Sqlite> for NonEmptyString {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> diesel::serialize::Result {
            <String as ToSql<Text, Sqlite>>::to_sql(&self.0, out)
        }
    }

    #[cfg(feature = "diesel")]
    #[test]
    fn diesel_compatible() {
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
                string: non_empty_string!("string"),
                optional_string: Some(non_empty_string!("string")),
            })
            .execute(&mut conn)
            .unwrap();

        assert_eq!(n, 1);
    }
}
