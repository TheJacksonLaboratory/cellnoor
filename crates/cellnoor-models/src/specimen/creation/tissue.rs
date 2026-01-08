use macro_attributes::{base_model, simple_enum};

use crate::specimen::{
    common::SpecimenCommonFields,
    variable::{Fixative, SpecimenType, SpecimenVariableFields, ThermalPreservationMethod},
};

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum TissueFixative {
    DithiobisSuccinimidylpropionate,
}

#[base_model]
#[derive(serde::Deserialize)]
#[serde(tag = "preservation_method")]
pub enum TissueCreation {
    ControlledRateFreezing {
        #[serde(flatten)]
        inner: SpecimenCommonFields,
    },
    FlashFreezing {
        #[serde(flatten)]
        inner: SpecimenCommonFields,
    },
    Fixation {
        #[serde(flatten)]
        inner: SpecimenCommonFields,
        fixative: TissueFixative,
    },
}

impl TissueCreation {
    pub fn inner(&self) -> &SpecimenCommonFields {
        match self {
            Self::ControlledRateFreezing { inner, .. }
            | Self::FlashFreezing { inner, .. }
            | Self::Fixation { inner, .. } => inner,
        }
    }

    fn thermal_preservation_method(&self) -> Option<ThermalPreservationMethod> {
        match &self {
            Self::ControlledRateFreezing { .. } => {
                Some(ThermalPreservationMethod::ControlledRateFreezing)
            }
            Self::Fixation { .. } => None,
            Self::FlashFreezing { .. } => Some(ThermalPreservationMethod::FlashFreezing),
        }
    }

    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let thermal_preservation_method = self.thermal_preservation_method();
        let (inner, fixative) = match self {
            Self::ControlledRateFreezing { inner } | Self::FlashFreezing { inner } => (inner, None),
            Self::Fixation { inner, fixative } => (inner, Some(Fixative::Tissue(fixative))),
        };

        (
            inner,
            SpecimenVariableFields {
                type_: SpecimenType::Suspension,
                embedded_in: None,
                fixative,
                thermal_preservation_method,
            },
        )
    }
}
