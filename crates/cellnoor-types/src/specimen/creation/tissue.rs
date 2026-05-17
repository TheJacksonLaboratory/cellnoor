use macro_attributes::base_model;

use crate::specimen::{
    Fixative, SpecimenType, ThermalPreservationMethod,
    creation::{NewSpecimenCommonFields, SpecimenInsertion},
};

#[base_model]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "snake_case", tag = "preservation_state")
)]
pub enum NewTissue {
    Fixed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewSpecimenCommonFields,
        fixative: Fixative,
    },
    Fresh {
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewSpecimenCommonFields,
    },
    ThermallyPreserved {
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewSpecimenCommonFields,
        thermal_preservation_method: ThermalPreservationMethod,
    },
}

impl NewTissue {
    fn into_common(self) -> NewSpecimenCommonFields {
        match self {
            Self::Fixed {
                common,
                fixative: _,
            }
            | Self::Fresh { common }
            | Self::ThermallyPreserved {
                common,
                thermal_preservation_method: _,
            } => common,
        }
    }

    fn fixative(&self) -> Option<Fixative> {
        match self {
            Self::Fixed {
                common: _,
                fixative,
            } => Some(*fixative),
            Self::Fresh { common: _ }
            | Self::ThermallyPreserved {
                common: _,
                thermal_preservation_method: _,
            } => None,
        }
    }

    fn thermal_preservation_method(&self) -> Option<ThermalPreservationMethod> {
        match self {
            Self::Fixed { .. } | Self::Fresh { .. } => None,
            Self::ThermallyPreserved {
                common: _,
                thermal_preservation_method,
                ..
            } => Some(*thermal_preservation_method),
        }
    }

    #[must_use]
    pub(super) fn split_for_insertion(self) -> SpecimenInsertion {
        let type_ = SpecimenType::Tissue;
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
