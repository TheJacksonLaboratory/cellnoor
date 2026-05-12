use macro_attributes::{base_model, unit_enum};

use crate::specimen::{
    NewSpecimenCommonFields,
    creation::{NewSpecimenRecord, SpecimenInsertion},
    record::ThermalPreservationMethod,
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

        let NewSpecimenCommonFields {
            readable_id,
            name,
            submitted_by,
            received_at,
            project_id,
            species,
            host_species,
            returned_by,
            returned_at,
            tissue,
            additional_data,
            measurements,
        } = self.into_common();

        (NewSpecimenRecord, measurements)
    }
}
