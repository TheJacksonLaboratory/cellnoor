#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Integer))]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(as = "u32"))]
pub struct RangedU16<const MIN: u16, const MAX: u16>(deranged::RangedU16<MIN, MAX>);

impl<const MIN: u16, const MAX: u16> RangedU16<MIN, MAX> {
    #[must_use]
    pub fn new(n: u16) -> Option<Self> {
        deranged::RangedU16::new(n).map(Self)
    }
}

#[cfg(feature = "diesel")]
mod diesel_impls {
    use diesel::{
        pg::Pg,
        serialize::{Output, ToSql},
        sql_types::Integer,
    };

    use super::RangedU16;

    impl<const MIN: u16, const MAX: u16> ToSql<Integer, Pg> for RangedU16<MIN, MAX> {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
            let as_int = self.0.get().into();
            <i32 as ToSql<Integer, Pg>>::to_sql(&as_int, &mut out.reborrow())
        }
    }
}
