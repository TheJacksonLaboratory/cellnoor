use macro_attributes::{base_model, simple_enum};

use crate::specimen::{
    common::SpecimenCommonFields,
    creation::SpecimenInsertion,
    variable::{Fixative, SpecimenType, SpecimenVariableFields, ThermalPreservationMethod},
};

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum SuspensionThermalPreservation {
    ControlledRateFreezing,
}

impl From<SuspensionThermalPreservation> for ThermalPreservationMethod {
    fn from(suspension_thermal_preservation: SuspensionThermalPreservation) -> Self {
        match suspension_thermal_preservation {
            SuspensionThermalPreservation::ControlledRateFreezing => Self::ControlledRateFreezing,
        }
    }
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case", tag = "preservation_state")]
pub enum NewSuspensionSpecimen {
    Fixed {
        #[serde(flatten)]
        inner: SpecimenCommonFields,
        fixative: Fixative,
    },
    Fresh(SpecimenCommonFields),
    ThermallyPreserved {
        #[serde(flatten)]
        inner: SpecimenCommonFields,
        thermal_preservation_method: SuspensionThermalPreservation,
    },
}

impl NewSuspensionSpecimen {
    pub(super) fn common(&self) -> &SpecimenCommonFields {
        match self {
            Self::Fixed { inner, fixative: _ }
            | Self::Fresh(inner)
            | Self::ThermallyPreserved {
                inner,
                thermal_preservation_method: _,
            } => inner,
        }
    }

    fn into_common(self) -> SpecimenCommonFields {
        match self {
            Self::Fixed { inner, fixative: _ }
            | Self::Fresh(inner)
            | Self::ThermallyPreserved {
                inner,
                thermal_preservation_method: _,
            } => inner,
        }
    }

    fn fixative(&self) -> Option<Fixative> {
        match self {
            Self::Fixed { inner: _, fixative } => Some(*fixative),
            Self::Fresh(_)
            | Self::ThermallyPreserved {
                inner: _,
                thermal_preservation_method: _,
            } => None,
        }
    }

    fn thermal_preservation_method(&self) -> Option<SuspensionThermalPreservation> {
        match self {
            Self::Fixed { .. } | Self::Fresh { .. } => None,
            Self::ThermallyPreserved {
                inner: _,
                thermal_preservation_method,
                ..
            } => Some(*thermal_preservation_method),
        }
    }

    pub fn split_for_insertion(self) -> SpecimenInsertion {
        let fixative = self.fixative();
        let thermal_preservation_method = self.thermal_preservation_method();

        (
            self.into_common(),
            SpecimenVariableFields {
                type_: SpecimenType::Suspension,
                embedded_in: None,
                fixative,
                thermal_preservation_method: thermal_preservation_method.map(Into::into),
            },
        )
    }
}
