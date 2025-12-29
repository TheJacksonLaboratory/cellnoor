use macro_attributes::{base_model, simple_enum};

use crate::specimen::common::{
    Fixative, SpecimenCommonFields, SpecimenType, SpecimenVariableFields,
};

const TYPE: SpecimenType = SpecimenType::Suspension;

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct CryopreservedSuspensionCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
}

impl CryopreservedSuspensionCreation {
    #[must_use]
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
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct FixedSuspensionCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
    fixative: SuspensionFixative,
}

impl FixedSuspensionCreation {
    #[must_use]
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self { inner, fixative } = self;

        (
            inner,
            SpecimenVariableFields {
                type_: TYPE,
                embedded_in: None,
                fixative: Some(Fixative::Suspension(fixative)),
                frozen: false,
                cryopreserved: false,
            },
        )
    }
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct FreshSuspensionCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
}

impl FreshSuspensionCreation {
    #[must_use]
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self { inner } = self;

        (
            inner,
            SpecimenVariableFields {
                type_: TYPE,
                embedded_in: None,
                fixative: None,
                frozen: false,
                cryopreserved: false,
            },
        )
    }
}

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum SuspensionFixative {
    DithiobisSuccinimidylpropionate,
    FormaldehydeDerivative,
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct FrozenSuspensionCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
}

impl FrozenSuspensionCreation {
    #[must_use]
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
