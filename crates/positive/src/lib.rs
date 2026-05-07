#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "postgres-types",
    derive(postgres_types::FromSql, postgres_types::ToSql)
)]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "schemars", schemars(with = "f32"))]
#[cfg_attr(feature = "postgres-types", postgres(transparent))]
pub struct PositiveF32(f32);

impl PartialEq<f32> for PositiveF32 {
    fn eq(&self, other: &f32) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<f32> for PositiveF32 {
    fn partial_cmp(&self, other: &f32) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "postgres-types",
    derive(postgres_types::FromSql, postgres_types::ToSql)
)]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "schemars", schemars(with = "i32"))]
#[cfg_attr(feature = "postgres-types", postgres(transparent))]
pub struct PositiveI32(i32);

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::Deserialize;

    use super::{PositiveF32, PositiveI32};

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
                    &format!("a positive float").as_str(),
                ));
            }

            Ok(Self(num))
        }
    }

    impl<'de> Deserialize<'de> for PositiveI32 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let num = i32::deserialize(deserializer)?;

            if num <= 0 {
                use serde::de;

                return Err(de::Error::invalid_value(
                    de::Unexpected::Float(f64::from(num)),
                    &format!("a positive integer").as_str(),
                ));
            }

            Ok(Self(num))
        }
    }
}
