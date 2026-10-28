//! Numbers that are greater than zero.

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
            &format!("a number n such that 0 < n <= {N}").as_str(),
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

/// A number greater than zero.
pub type PositiveF32 = PositiveBounded<f32, { u32::MAX }>;

/// A number greater than zero.
pub type PositiveI32 = PositiveBounded<i32, { u32::MAX }>;

/// A number in `0 < n <= N`.
pub type PositiveBoundedF32<const N: u32> = PositiveBounded<f32, N>;

#[cfg(test)]
mod tests {
    use crate::positive::{PositiveBoundedF32, PositiveF32};

    #[test]
    fn new_rejects_values_outside_the_bounds() {
        assert!(PositiveF32::new(1.0).is_some());
        assert!(PositiveF32::new(0.0).is_none());
        assert!(PositiveF32::new(-1.0).is_none());

        assert!(PositiveBoundedF32::<10>::new(10.0).is_some());
        assert!(PositiveBoundedF32::<10>::new(10.5).is_none());
    }
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod serde_tests {
    use pretty_assertions::assert_eq;

    use crate::positive::PositiveBoundedF32;

    #[test]
    fn deserialize_out_of_bounds_number_fails() {
        let result: Result<Vec<PositiveBoundedF32<10>>, _> = serde_json::from_str("[10.5]");

        assert!(result.is_err());
    }

    #[test]
    fn deserialize_in_bounds_number_succeeds() {
        let deserialized: [PositiveBoundedF32<10>; 1] = serde_json::from_str("[10.0]").unwrap();

        assert_eq!(deserialized, [PositiveBoundedF32::<10>::new(10.0).unwrap()]);
    }
}
