use macro_attributes::{base_model, unit_enum};

use crate::specimen::{
    Fixative, SpecimenType, ThermalPreservationMethod,
    creation::{NewSpecimenCommonFields, SpecimenInsertion},
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
        inner: NewSpecimenCommonFields,
        fixative: Option<BlockFixative>,
    },
    CarboxymethylCellulose {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: NewSpecimenCommonFields,
        fixative: Option<BlockFixative>,
    },
    Paraffin {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: NewSpecimenCommonFields,
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
    fn fixative(&self) -> Option<Fixative> {
        let fixative = match self {
            Self::CarboxymethylCellulose { fixative, .. } => *fixative,
            Self::OptimalCuttingTemperatureCompound { fixative, .. } => *fixative,
            Self::Paraffin { fixative, .. } => Some(*fixative),
        };

        fixative.map(Into::into)
    }

    fn embedded_in(&self) -> BlockEmbeddingMatrix {
        match self {
            Self::CarboxymethylCellulose { .. } => BlockEmbeddingMatrix::CarboxymethylCellulose,
            Self::OptimalCuttingTemperatureCompound { .. } => {
                BlockEmbeddingMatrix::OptimalCuttingTemperatureCompound
            }
            Self::Paraffin { .. } => BlockEmbeddingMatrix::Paraffin,
        }
    }

    fn thermal_preservation_method(&self) -> Option<ThermalPreservationMethod> {
        match self {
            Self::CarboxymethylCellulose { .. }
            | Self::OptimalCuttingTemperatureCompound { .. } => {
                Some(ThermalPreservationMethod::FlashFreezing)
            }
            Self::Paraffin { .. } => None,
        }
    }

    fn into_inner(self) -> NewSpecimenCommonFields {
        match self {
            Self::CarboxymethylCellulose { inner, .. }
            | Self::OptimalCuttingTemperatureCompound { inner, .. }
            | Self::Paraffin { inner, .. } => inner,
        }
    }

    pub(super) fn split_for_insertion(self) -> SpecimenInsertion {
        let type_ = SpecimenType::Block;
        let embedded_in = self.embedded_in();
        let fixative = self.fixative();
        let thermal_preservation_method = self.thermal_preservation_method();

        SpecimenInsertion::from_fields(
            self.into_inner(),
            type_,
            Some(embedded_in),
            fixative,
            thermal_preservation_method,
        )
    }
}
