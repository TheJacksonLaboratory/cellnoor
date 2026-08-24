use macro_attributes::{base_model, discriminant_unit_enum};

use crate::specimen::creation::common::{FlashFreezing, FormaldehydeDerivative};

#[base_model]
#[derive(Copy, strum::EnumDiscriminants)]
#[strum_discriminants(name(BlockEmbeddingMatrix), discriminant_unit_enum)]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "snake_case", tag = "embedded_in")
)]
pub enum BlockFields {
    CarboxymethylCellulose {
        fixative: Option<FormaldehydeDerivative>,
        thermal_preservation_method: FlashFreezing,
    },
    OptimalCuttingTemperatureCompound {
        fixative: Option<FormaldehydeDerivative>,
        thermal_preservation_method: FlashFreezing,
    },
    Paraffin {
        fixative: FormaldehydeDerivative,
    },
}
