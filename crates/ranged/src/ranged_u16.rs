#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "schemars", schemars(with = "u32"))]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Integer))]
pub struct RangedU16<const MIN: u16, const MAX: u16>(deranged::RangedU16<MIN, MAX>);

impl<const MIN: u16, const MAX: u16> RangedU16<MIN, MAX> {
    #[must_use]
    pub fn new(n: u16) -> Option<Self> {
        deranged::RangedU16::new(n).map(Self)
    }
}

impl<const MIN: u16, const MAX: u16> From<RangedU16<MIN, MAX>> for u16 {
    fn from(value: RangedU16<MIN, MAX>) -> u16 {
        value.0.into()
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

    use super::RangedU16;

    impl<const MIN: u16, const MAX: u16> FromSql<Integer, Pg> for RangedU16<MIN, MAX> {
        fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
            let as_int = <i32 as FromSql<Integer, Pg>>::from_sql(bytes)?;

            Ok(deranged::RangedU16::<MIN, MAX>::new(as_int as u16)
                .map(RangedU16)
                .unwrap())
        }
    }

    impl<const MIN: u16, const MAX: u16> ToSql<Integer, Pg> for RangedU16<MIN, MAX> {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
            let as_int = self.0.get().into();
            <i32 as ToSql<Integer, Pg>>::to_sql(&as_int, &mut out.reborrow())
        }
    }
}
