#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::BigInt))]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(as = "u32"))]
pub struct RangedU32<const MIN: u32, const MAX: u32>(deranged::RangedU32<MIN, MAX>);

impl<const MIN: u32, const MAX: u32> RangedU32<MIN, MAX> {
    #[must_use]
    pub fn new(n: u32) -> Option<Self> {
        deranged::RangedU32::new(n).map(Self)
    }
}

#[cfg(feature = "diesel")]
mod diesel_impls {
    use diesel::{
        pg::Pg,
        serialize::{Output, ToSql},
        sql_types::BigInt,
    };

    use super::RangedU32;

    impl<const MIN: u32, const MAX: u32> ToSql<BigInt, Pg> for RangedU32<MIN, MAX> {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
            let as_int = self.0.get().into();
            <i64 as ToSql<BigInt, Pg>>::to_sql(&as_int, &mut out.reborrow())
        }
    }
}
