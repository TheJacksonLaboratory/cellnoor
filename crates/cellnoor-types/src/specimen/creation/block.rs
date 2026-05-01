use macro_attributes::{base_model, unit_enum};

use crate::specimen::{
    Fixative, SpecimenCommonFields, SpecimenType, SpecimenVariableFields,
    ThermalPreservationMethod, creation::SpecimenInsertion,
};

#[unit_enum]
pub enum BlockFixative {
    FormaldehydeDerivative,
}

impl From<BlockFixative> for Fixative {
    fn from(block_fixative: BlockFixative) -> Self {
        match block_fixative {
            BlockFixative::FormaldehydeDerivative => Fixative::FormaldehydeDerivative,
        }
    }
}

#[base_model]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "snake_case", tag = "embedded_in")
)]
pub enum NewBlock {
    OptimalCuttingTemperatureCompound {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: SpecimenCommonFields,
        fixative: Option<BlockFixative>,
    },
    CarboxymethylCellulose {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: SpecimenCommonFields,
        fixative: Option<BlockFixative>,
    },
    Paraffin {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: SpecimenCommonFields,
        fixative: BlockFixative,
    },
}

#[unit_enum]
pub enum BlockEmbeddingMatrix {
    CarboxymethylCellulose,
    OptimalCuttingTemperatureCompound,
    Paraffin,
}

impl NewBlock {
    pub(super) fn common(&self) -> &SpecimenCommonFields {
        match self {
            Self::CarboxymethylCellulose { inner, .. }
            | Self::OptimalCuttingTemperatureCompound { inner, .. }
            | Self::Paraffin { inner, .. } => inner,
        }
    }

    fn into_common(self) -> SpecimenCommonFields {
        match self {
            Self::CarboxymethylCellulose { inner, fixative: _ }
            | Self::OptimalCuttingTemperatureCompound { inner, fixative: _ }
            | Self::Paraffin { inner, fixative: _ } => inner,
        }
    }

    fn fixative(&self) -> Option<BlockFixative> {
        match self {
            Self::CarboxymethylCellulose { inner: _, fixative }
            | Self::OptimalCuttingTemperatureCompound { inner: _, fixative } => *fixative,
            Self::Paraffin { inner: _, fixative } => Some(*fixative),
        }
    }

    fn embedding_matrix(&self) -> BlockEmbeddingMatrix {
        match &self {
            Self::CarboxymethylCellulose { .. } => BlockEmbeddingMatrix::CarboxymethylCellulose,
            Self::OptimalCuttingTemperatureCompound { .. } => {
                BlockEmbeddingMatrix::OptimalCuttingTemperatureCompound
            }
            Self::Paraffin { .. } => BlockEmbeddingMatrix::Paraffin,
        }
    }

    fn thermal_preservation_method(&self) -> Option<ThermalPreservationMethod> {
        match &self {
            Self::CarboxymethylCellulose { .. } => Some(ThermalPreservationMethod::FlashFreezing),
            Self::OptimalCuttingTemperatureCompound { .. } => {
                Some(ThermalPreservationMethod::FlashFreezing)
            }
            Self::Paraffin { .. } => None,
        }
    }

    pub fn split_for_insertion(self) -> SpecimenInsertion {
        let fixative = self.fixative();
        let embedded_in = Some(self.embedding_matrix());
        let thermal_preservation_method = self.thermal_preservation_method();

        (
            self.into_common(),
            SpecimenVariableFields {
                type_: SpecimenType::Block,
                embedded_in,
                fixative: fixative.map(Into::into),
                thermal_preservation_method,
            },
        )
    }
}
