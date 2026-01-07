use macro_attributes::{base_model, simple_enum};

use crate::specimen::common::{
    Fixative, PreservationMethod, SpecimenCommonFields, SpecimenType, SpecimenVariableFields,
    preservation_methods_from_fixative_and_flash_frozen,
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
                preservation_methods: vec![Some(PreservationMethod::Cryopreservation)],
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
pub struct NonCryopreservedSuspensionCreation {
    #[serde(flatten)]
    pub(super) inner: SpecimenCommonFields,
    fixative: Option<SuspensionFixative>,
    flash_frozen: bool,
}

impl NonCryopreservedSuspensionCreation {
    #[must_use]
    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let Self {
            inner,
            fixative,
            flash_frozen,
        } = self;

        let preservation_methods =
            preservation_methods_from_fixative_and_flash_frozen(fixative, flash_frozen);

        (
            inner,
            SpecimenVariableFields {
                type_: TYPE,
                embedded_in: None,
                fixative: fixative.map(Fixative::Suspension),
                preservation_methods,
            },
        )
    }
}
