use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Float))]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub struct PositiveF32(pub f32);

impl Display for PositiveF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::Deserialize;

    use super::PositiveF32;

    impl<'de> Deserialize<'de> for PositiveF32 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let num = f32::deserialize(deserializer)?;

            if num <= 0.0 {
                use serde::de;

                return Err(de::Error::invalid_value(
                    de::Unexpected::Float(f64::from(num)),
                    &"a positive float",
                ));
            }

            Ok(Self(num))
        }
    }
}

#[cfg(feature = "diesel")]
mod diesel_impls {
    use diesel::{
        deserialize::FromSql,
        pg::{Pg, PgValue},
        serialize::{Output, ToSql},
        sql_types::Float,
    };

    use super::PositiveF32;

    impl FromSql<Float, Pg> for PositiveF32 {
        fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {
            <f32 as FromSql<Float, Pg>>::from_sql(bytes).map(Self)
        }
    }

    impl ToSql<Float, Pg> for PositiveF32 {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
            <f32 as ToSql<Float, Pg>>::to_sql(&self.0, out)
        }
    }
}
