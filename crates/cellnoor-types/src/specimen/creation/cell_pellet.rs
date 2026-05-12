use macro_attributes::{base_model, unit_enum};

use crate::specimen::{
    NewSpecimenCommonFields, NewSpecimenVariableFields, SpecimenType, ThermalPreservationMethod,
    creation::SpecimenInsertion,
};

#[unit_enum]
pub enum CellPelletThermalPreservation {
    FlashFreezing,
}

impl From<CellPelletThermalPreservation> for ThermalPreservationMethod {
    fn from(pellet_thermal_preservation: CellPelletThermalPreservation) -> Self {
        match pellet_thermal_preservation {
            CellPelletThermalPreservation::FlashFreezing => Self::FlashFreezing,
        }
    }
}

#[base_model]
pub struct NewCellPellet {
    #[cfg_attr(feature = "serde", serde(flatten))]
    inner: NewSpecimenCommonFields,
    thermal_preservation_method: CellPelletThermalPreservation,
}

impl NewCellPellet {
    fn into_common(self) -> NewSpecimenCommonFields {
        self.inner
    }

    pub fn split_for_insertion(self) -> SpecimenInsertion {
        let thermal_preservation_method = self.thermal_preservation_method.into();

        (
            self.into_common().split_for_insertion(),
            NewSpecimenVariableFields {
                type_: SpecimenType::CellPellet,
                embedded_in: None,
                fixative: None,
                thermal_preservation_method: Some(thermal_preservation_method),
            },
        )
    }
}
