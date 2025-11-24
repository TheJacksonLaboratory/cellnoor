use macro_attributes::{base_model, simple_enum};

use crate::specimen::common::{
    EmbeddingMatrix, Fixative, SpecimenCommonFields, SpecimenType, SpecimenVariableFields,
};

const TYPE: SpecimenType = SpecimenType::Tissue;

#[base_model]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct CryopreservedTissueCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
}

impl CryopreservedTissueCreation {
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self { inner } = self;

        (
            inner,
            SpecimenVariableFields {
                type_: TYPE,
                embedded_in: None,
                fixative: None,
                frozen: false,
                cryopreserved: true,
            },
        )
    }
}

#[base_model]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct FixedTissueCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
    fixative: TissueFixative,
}

impl FixedTissueCreation {
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self { inner, fixative } = self;

        (
            inner,
            SpecimenVariableFields {
                type_: TYPE,
                embedded_in: None,
                fixative: Some(Fixative::Tissue(fixative)),
                frozen: false,
                cryopreserved: false,
            },
        )
    }
}

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum TissueFixative {
    DithiobisSuccinimidylpropionate,
}

#[base_model]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct FrozenTissueCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
}

impl FrozenTissueCreation {
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self { inner } = self;

        (
            inner,
            SpecimenVariableFields {
                type_: TYPE,
                embedded_in: None,
                fixative: None,
                frozen: true,
                cryopreserved: false,
            },
        )
    }
}
