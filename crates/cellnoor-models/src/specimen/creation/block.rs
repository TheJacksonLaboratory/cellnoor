use macro_attributes::{base_model, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};

use crate::specimen::{
    common::SpecimenCommonFields,
    variable::{Fixative, SpecimenType, SpecimenVariableFields, ThermalPreservationMethod},
};
#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum BlockFixative {
    FormaldehydeDerivative,
}

#[base_model]
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "embedded_in")]
pub enum BlockCreation {
    OptimalCuttingTemperatureCompound {
        #[serde(flatten)]
        inner: SpecimenCommonFields,
        fixative: Option<BlockFixative>,
    },
    CarboxymethylCellulose {
        #[serde(flatten)]
        inner: SpecimenCommonFields,
        fixative: Option<BlockFixative>,
    },
    Paraffin {
        #[serde(flatten)]
        inner: SpecimenCommonFields,
        fixative: BlockFixative,
    },
}

#[simple_enum]
pub enum BlockEmbeddingMatrix {
    CarboxymethylCellulose,
    OptimalCuttingTemperatureCompound,
    Paraffin,
}

#[cfg(feature = "app")]
impl EnumFromSql for BlockEmbeddingMatrix {}
impl_enum_from_sql!(BlockEmbeddingMatrix);

#[cfg(feature = "app")]
impl EnumToSql for BlockEmbeddingMatrix {}
impl_enum_to_sql!(BlockEmbeddingMatrix);

impl BlockCreation {
    pub fn inner(&self) -> &SpecimenCommonFields {
        match self {
            Self::CarboxymethylCellulose { inner, .. }
            | Self::OptimalCuttingTemperatureCompound { inner, .. }
            | Self::Paraffin { inner, .. } => inner,
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

    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let embedded_in = Some(self.embedding_matrix());
        let thermal_preservation_method = self.thermal_preservation_method();

        let (inner, fixative) = match self {
            Self::CarboxymethylCellulose { inner, fixative }
            | Self::OptimalCuttingTemperatureCompound { inner, fixative } => (inner, fixative),
            Self::Paraffin { inner, fixative } => (inner, Some(fixative)),
        };

        (
            inner,
            SpecimenVariableFields {
                type_: SpecimenType::Block,
                embedded_in,
                fixative: fixative.map(Fixative::Block),
                thermal_preservation_method,
            },
        )
    }
}
