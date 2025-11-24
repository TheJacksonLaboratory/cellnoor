use macro_attributes::{base_model, simple_enum};

use crate::specimen::common::{
    EmbeddingMatrix, Fixative, SpecimenCommonFields, SpecimenType, SpecimenVariableFields,
};

#[base_model]
pub struct FixedBlockCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
    embedded_in: FixedBlockEmbeddingMatrix,
    fixative: BlockFixative,
}

impl FixedBlockCreation {
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self {
            inner,
            embedded_in,
            fixative,
        } = self;

        (
            inner,
            SpecimenVariableFields {
                type_: SpecimenType::Block,
                embedded_in: Some(EmbeddingMatrix::FixedBlock(embedded_in)),
                fixative: Some(Fixative::Block(fixative)),
                frozen: false,
                cryopreserved: false,
            },
        )
    }
}

#[simple_enum]
pub enum FixedBlockEmbeddingMatrix {
    Paraffin,
}

#[simple_enum]
pub enum BlockFixative {
    FormaldehydeDerivative,
}

#[base_model]
pub struct FrozenBlockCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
    embedded_in: FrozenBlockEmbeddingMatrix,
    fixative: Option<BlockFixative>,
}

impl FrozenBlockCreation {
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self {
            inner,
            embedded_in,
            fixative,
        } = self;

        (
            inner,
            SpecimenVariableFields {
                type_: SpecimenType::Block,
                embedded_in: Some(EmbeddingMatrix::FrozenBlock(embedded_in)),
                fixative: fixative.map(Fixative::Block),
                frozen: true,
                cryopreserved: false,
            },
        )
    }
}

#[simple_enum]
pub enum FrozenBlockEmbeddingMatrix {
    CarboxymethylCellulose,
    OptimalCuttingTemperatureCompound,
}
