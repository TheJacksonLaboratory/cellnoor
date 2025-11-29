use macro_attributes::{base_model, simple_enum};

use crate::specimen::common::{
    EmbeddingMatrix, Fixative, SpecimenCommonFields, SpecimenType, SpecimenVariableFields,
};

const TYPE: SpecimenType = SpecimenType::Block;

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct FixedBlockCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
    embedded_in: FixedBlockEmbeddingMatrix,
    fixative: BlockFixative,
}

impl FixedBlockCreation {
    #[must_use]
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self {
            inner,
            embedded_in,
            fixative,
        } = self;

        (
            inner,
            SpecimenVariableFields {
                type_: TYPE,
                embedded_in: Some(EmbeddingMatrix::FixedBlock(embedded_in)),
                fixative: Some(Fixative::Block(fixative)),
                frozen: false,
                cryopreserved: false,
            },
        )
    }
}

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum FixedBlockEmbeddingMatrix {
    Paraffin,
}

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum BlockFixative {
    FormaldehydeDerivative,
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct FrozenBlockCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
    embedded_in: FrozenBlockEmbeddingMatrix,
    fixative: Option<BlockFixative>,
}

impl FrozenBlockCreation {
    #[must_use]
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self {
            inner,
            embedded_in,
            fixative,
        } = self;

        (
            inner,
            SpecimenVariableFields {
                type_: TYPE,
                embedded_in: Some(EmbeddingMatrix::FrozenBlock(embedded_in)),
                fixative: fixative.map(Fixative::Block),
                frozen: true,
                cryopreserved: false,
            },
        )
    }
}

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum FrozenBlockEmbeddingMatrix {
    CarboxymethylCellulose,
    OptimalCuttingTemperatureCompound,
}
