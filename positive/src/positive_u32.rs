#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Integer))]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub struct PositiveU32(u32);

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::Deserialize;

    use super::PositiveU32;

    impl<'de> Deserialize<'de> for PositiveU32 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let num = u32::deserialize(deserializer)?;

            if num == 0 {
                use serde::de;

                return Err(de::Error::invalid_value(
                    de::Unexpected::Unsigned(u64::from(num)),
                    &"a positive integer",
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
        sql_types::Integer,
    };

    use super::PositiveU32;

    impl FromSql<Integer, Pg> for PositiveU32 {
        fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {
            let num = <i32 as FromSql<Integer, Pg>>::from_sql(bytes).map(u32::try_from)??;

            Ok(Self(num))
        }
    }

    impl ToSql<Integer, Pg> for PositiveU32 {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
            let as_int = i32::try_from(self.0)?;
            <i32 as ToSql<Integer, Pg>>::to_sql(&as_int, &mut out.reborrow())
        }
    }
}
