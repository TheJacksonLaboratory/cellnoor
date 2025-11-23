use jiff::Timestamp;
use macro_attributes::{insert, simple_enum};
#[cfg(feature = "app")]
use scamplers_schema::specimens;

use crate::specimen::common::Fields;

#[simple_enum]
pub enum FixedBlockEmbeddingMatrix {
    Paraffin,
}

#[simple_enum]
pub enum BlockFixative {
    FormaldehydeDerivative,
}

#[simple_enum]
#[derive(Default)]
pub enum Type {
    #[default]
    Block,
}

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = specimens))]
pub struct Fixed {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: Fields,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::Timestamp,
        deserialize_as = jiff_diesel::Timestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(type = "Temporal.Instant"))]
    received_at: Timestamp,
    #[serde(skip)]
    type_: Type,
    embedded_in: FixedBlockEmbeddingMatrix,
    fixative: BlockFixative,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::NullableTimestamp,
        deserialize_as = jiff_diesel::NullableTimestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(type = "Temporal.Instant"))]
    returned_at: Option<Timestamp>,
}

#[simple_enum]
pub enum FrozenBlockEmbeddingMatrix {
    CarboxymethylCellulose,
    OptimalCuttingTemperatureCompound,
}

const TRUE: bool = true;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = specimens))]
pub struct Frozen {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: Fields,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::Timestamp,
        deserialize_as = jiff_diesel::Timestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(type = "Temporal.Instant"))]
    received_at: Timestamp,
    #[serde(skip)]
    type_: Type,
    embedded_in: FrozenBlockEmbeddingMatrix,
    fixative: Option<BlockFixative>,
    #[serde(skip, default = "crate::specimen::common::true_")]
    frozen: bool,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::NullableTimestamp,
        deserialize_as = jiff_diesel::NullableTimestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(type = "Temporal.Instant"))]
    returned_at: Option<Timestamp>,
}
