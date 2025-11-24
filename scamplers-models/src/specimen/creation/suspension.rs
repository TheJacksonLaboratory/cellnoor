use macro_attributes::{base_model, simple_enum};

use crate::specimen::common::{
    Fixative, SpecimenCommonFields, SpecimenType, SpecimenVariableFields,
};

const TYPE: SpecimenType = SpecimenType::Suspension;

#[base_model]
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
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct FixedOrFreshSuspensionCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
    fixative: Option<SuspensionFixative>,
}

impl FixedOrFreshSuspensionCreation {
    #[must_use]
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self { inner, fixative } = self;

        (
            inner,
            SpecimenVariableFields {
                type_: TYPE,
                embedded_in: None,
                fixative: fixative.map(Fixative::Suspension),
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
