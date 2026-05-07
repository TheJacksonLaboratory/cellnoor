use macro_attributes::base_model;

use crate::specimen::{
    Fixative, SpecimenCommonFields, SpecimenType, SpecimenVariableFields,
    ThermalPreservationMethod, creation::SpecimenInsertion,
};

#[base_model]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "snake_case", tag = "preservation_state")
)]
pub enum NewTissue {
    Fixed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: SpecimenCommonFields,
        fixative: Fixative,
    },
    Fresh {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: SpecimenCommonFields,
    },
    ThermallyPreserved {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: SpecimenCommonFields,
        thermal_preservation_method: ThermalPreservationMethod,
    },
}

impl NewTissue {
    fn into_common(self) -> SpecimenCommonFields {
        match self {
            Self::Fixed { inner, fixative: _ }
            | Self::Fresh { inner }
            | Self::ThermallyPreserved {
                inner,
                thermal_preservation_method: _,
            } => inner,
        }
    }

    fn fixative(&self) -> Option<Fixative> {
        match self {
            Self::Fixed { inner: _, fixative } => Some(*fixative),
            Self::Fresh { inner: _ }
            | Self::ThermallyPreserved {
                inner: _,
                thermal_preservation_method: _,
            } => None,
        }
    }

    fn thermal_preservation_method(&self) -> Option<ThermalPreservationMethod> {
        match self {
            Self::Fixed { .. } | Self::Fresh { .. } => None,
            Self::ThermallyPreserved {
                inner: _,
                thermal_preservation_method,
                ..
            } => Some(*thermal_preservation_method),
        }
    }

    #[must_use]
    pub fn split_for_insertion(self) -> SpecimenInsertion {
        let fixative = self.fixative();
        let thermal_preservation_method = self.thermal_preservation_method();

        (
            self.into_common().split_for_insertion(),
            SpecimenVariableFields {
                type_: SpecimenType::Suspension,
                embedded_in: None,
                fixative,
                thermal_preservation_method,
            },
        )
    }
}
