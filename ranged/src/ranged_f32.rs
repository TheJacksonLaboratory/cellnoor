use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Float))]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub struct RangedF32<const MIN: u32, const MAX: u32>(f32);

impl<const MIN: u32, const MAX: u32> Display for RangedF32<MIN, MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::Deserialize;

    use super::RangedF32;

    impl<'de, const MIN: u32, const MAX: u32> Deserialize<'de> for RangedF32<MIN, MAX> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let num = f32::deserialize(deserializer)?;

            if num < MIN as f32 || num > MAX as f32 {
                use serde::de;

                return Err(de::Error::invalid_value(
                    de::Unexpected::Float(f64::from(num)),
                    &format!("a float between {MIN} and {MAX}").as_str(),
                ));
            }

            Ok(Self(num))
        }
    }
}

#[cfg(feature = "diesel")]
mod diesel_impls {
    use diesel::{
        pg::Pg,
        serialize::{Output, ToSql},
        sql_types::Float,
    };

    use super::RangedF32;

    impl<const MIN: u32, const MAX: u32> ToSql<Float, Pg> for RangedF32<MIN, MAX> {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
            <f32 as ToSql<Float, Pg>>::to_sql(&self.0, out)
        }
    }
}
