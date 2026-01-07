use macro_attributes::{base_model, simple_enum};

use crate::specimen::common::{
    EmbeddingMatrix, Fixative, PreservationMethod, SpecimenCommonFields, SpecimenType,
    SpecimenVariableFields,
};

const TYPE: SpecimenType = SpecimenType::Block;

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum BlockFixative {
    FormaldehydeDerivative,
}

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum FixedBlockEmbeddingMatrix {
    Paraffin,
}

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum FlashFrozenBlockEmbeddingMatrix {
    CarboxymethylCellulose,
    OptimalCuttingTemperatureCompound,
}

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
                preservation_methods: vec![Some(PreservationMethod::Fixation)],
            },
        )
    }
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct FlashFrozenBlockCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
    embedded_in: FlashFrozenBlockEmbeddingMatrix,
    fixative: Option<BlockFixative>,
}

impl FlashFrozenBlockCreation {
    #[must_use]
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self {
            inner,
            embedded_in,
            fixative,
        } = self;

        let mut preservation_methods = vec![Some(PreservationMethod::FlashFreezing)];
        if fixative.is_some() {
            preservation_methods.push(Some(PreservationMethod::Fixation));
        }

        (
            inner,
            SpecimenVariableFields {
                type_: TYPE,
                embedded_in: Some(EmbeddingMatrix::FlashFrozenBlock(embedded_in)),
                fixative: fixative.map(Fixative::Block),
                preservation_methods,
            },
        )
    }
}
