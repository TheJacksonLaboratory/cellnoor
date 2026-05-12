use macro_attributes::{base_model, unit_enum};

use crate::{
    id::NoId,
    specimen::{
        NewSpecimenCommonFields,
        creation::{NewSpecimenRecord, SpecimenInsertion},
        record::{Fixative, SpecimenType, ThermalPreservationMethod},
    },
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
    pub(crate) fn split_for_insertion(self) -> SpecimenInsertion {
        let type_ = SpecimenType::Block;
        let (common, embedded_in, fixative, thermal_preservation_method) = match self {
            Self::CarboxymethylCellulose { inner, fixative } => (
                inner,
                BlockEmbeddingMatrix::CarboxymethylCellulose,
                fixative,
                Some(ThermalPreservationMethod::FlashFreezing),
            ),
            Self::OptimalCuttingTemperatureCompound { inner, fixative } => (
                inner,
                BlockEmbeddingMatrix::OptimalCuttingTemperatureCompound,
                fixative,
                Some(ThermalPreservationMethod::FlashFreezing),
            ),
            Self::Paraffin { inner, fixative } => {
                (inner, BlockEmbeddingMatrix::Paraffin, Some(fixative), None)
            }
        };

        SpecimenInsertion::from_common_and_variable(
            common,
            type_,
            Some(embedded_in),
            fixative,
            thermal_preservation_method,
        )
    }
}
