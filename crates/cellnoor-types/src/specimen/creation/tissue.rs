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
        inner: NewSpecimenCommonFields,
        fixative: Fixative,
    },
    Fresh {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: NewSpecimenCommonFields,
    },
    ThermallyPreserved {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: NewSpecimenCommonFields,
        thermal_preservation_method: ThermalPreservationMethod,
    },
}

impl NewTissue {
    fn into_inner(self) -> NewSpecimenCommonFields {
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
    pub(super) fn split_for_insertion(self) -> SpecimenInsertion {
        let _type_ = SpecimenType::Tissue;
        let fixative = self.fixative();
        let thermal_preservation_method = self.thermal_preservation_method();

        SpecimenInsertion::from_fields(
            self.into_inner(),
            SpecimenType::Tissue,
            None,
            fixative,
            thermal_preservation_method,
        )
    }
}
