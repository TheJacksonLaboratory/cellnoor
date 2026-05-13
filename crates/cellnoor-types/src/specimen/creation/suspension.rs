use macro_attributes::{base_model, unit_enum};

use crate::specimen::{
    Fixative, SpecimenType, ThermalPreservationMethod,
    creation::{NewSpecimenCommonFields, SpecimenInsertion},
};

#[unit_enum]
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
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "snake_case", tag = "preservation_state")
)]
pub enum NewSuspensionSpecimen {
    Fixed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: NewSpecimenCommonFields,
        fixative: Fixative,
    },
    Fresh(NewSpecimenCommonFields),
    ThermallyPreserved {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: NewSpecimenCommonFields,
        thermal_preservation_method: SuspensionThermalPreservation,
    },
}

impl NewSuspensionSpecimen {
    fn into_common(self) -> NewSpecimenCommonFields {
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

    pub(super) fn split_for_insertion(self) -> SpecimenInsertion {
        let type_ = SpecimenType::Suspension;
        let fixative = self.fixative();
        let thermal_preservation_method = self.thermal_preservation_method();

        SpecimenInsertion::from_fields(
            self.into_common(),
            type_,
            None,
            fixative,
            thermal_preservation_method,
        )
    }
}
