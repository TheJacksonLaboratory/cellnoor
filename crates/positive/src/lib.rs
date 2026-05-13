use crate::positive::{Positive, PositiveBounded};

mod positive {
    #[cfg(feature = "postgres-types")]
    use bytes::BytesMut;
    #[cfg(feature = "postgres-types")]
    use postgres_types::{FromSql, ToSql, to_sql_checked};
    #[cfg(feature = "serde")]
    use serde::{
        Deserialize,
        de::{self, Unexpected},
    };

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    #[cfg_attr(feature = "schemars", schemars(with = "T"))]
    pub struct PositiveBounded<T, const N: u32>(T);

    impl<T, const N: u32> PositiveBounded<T, N>
    where
        T: Copy + Into<f64>,
    {
        pub fn new(val: T) -> Option<Self> {
            let as_f64 = val.into();

            if as_f64 <= 0.0 || as_f64 > N.into() {
                return None;
            }

            Some(Self(val))
        }
    }

    impl<T, const N: u32> PartialEq<T> for PositiveBounded<T, N>
    where
        T: PartialEq,
    {
        fn eq(&self, other: &T) -> bool {
            self.0.eq(other)
        }
    }

    impl<T, const N: u32> PartialOrd<T> for PositiveBounded<T, N>
    where
        T: PartialOrd,
    {
        fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
            self.0.partial_cmp(other)
        }
    }

    #[cfg(feature = "serde")]
    impl<'de, T, const N: u32> Deserialize<'de> for PositiveBounded<T, N>
    where
        T: Copy + Into<f64> + Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let val = T::deserialize(deserializer)?;

            Self::new(val).ok_or(de::Error::invalid_value(
                Unexpected::Other(&format!("a number <= 0 or > {N}")),
                &format!("a number n such that 0 < 0 <= {N}").as_str(),
            ))
        }
    }

    #[cfg(feature = "postgres-types")]
    impl<'a, T, const N: u32> FromSql<'a> for PositiveBounded<T, N>
    where
        T: FromSql<'a>,
    {
        fn from_sql(
            ty: &postgres_types::Type,
            raw: &'a [u8],
        ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            T::from_sql(ty, raw).map(Self)
        }

        fn accepts(ty: &postgres_types::Type) -> bool {
            T::accepts(ty)
        }
    }

    #[cfg(feature = "postgres-types")]
    impl<T, const N: u32> ToSql for PositiveBounded<T, N>
    where
        T: ToSql,
    {
        to_sql_checked!();

        fn to_sql(
            &self,
            ty: &postgres_types::Type,
            out: &mut BytesMut,
        ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
        where
            Self: Sized,
        {
            self.0.to_sql(ty, out)
        }

        fn accepts(ty: &postgres_types::Type) -> bool
        where
            Self: Sized,
        {
            T::accepts(ty)
        }
    }

    pub type Positive<T> = PositiveBounded<T, { u32::MAX }>;
}

pub type PositiveF32 = Positive<f32>;

pub type PositiveU32 = Positive<u32>;

pub type PositiveBoundedF32<const N: u32> = PositiveBounded<f32, N>;

pub type PositiveBoundedU32<const N: u32> = PositiveBounded<u32, N>;
