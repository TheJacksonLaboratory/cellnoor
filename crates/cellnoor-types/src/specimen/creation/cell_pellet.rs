use macro_attributes::{base_model, unit_enum};

use crate::specimen::{
    Fixative, SpecimenType, ThermalPreservationMethod,
    creation::{NewSpecimenCommonFields, SpecimenInsertion},
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
    pub(crate) fn split_for_insertion(self) -> SpecimenInsertion {
        let thermal_preservation_method = self.thermal_preservation_method.into();

        SpecimenInsertion::from_fields(
            self.inner,
            SpecimenType::CellPellet,
            None,
            None::<Fixative>,
            thermal_preservation_method,
        )
    }
}
