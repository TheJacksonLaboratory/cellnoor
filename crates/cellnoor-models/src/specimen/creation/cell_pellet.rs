use macro_attributes::{base_model, simple_enum};

use crate::specimen::{
    common::SpecimenCommonFields,
    variable::{SpecimenType, SpecimenVariableFields, ThermalPreservationMethod},
};

#[simple_enum]
#[derive(strum::VariantArray)]
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
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(schemars::JsonSchema))]
pub struct NewCellPellet {
    #[serde(flatten)]
    inner: SpecimenCommonFields,
    thermal_preservation_method: CellPelletThermalPreservation,
}

impl NewCellPellet {
    pub(super) fn common(&self) -> &SpecimenCommonFields {
        &self.inner
    }

    fn into_common(self) -> SpecimenCommonFields {
        self.inner
    }

    pub fn split_for_insertion(self) -> (SpecimenCommonFields, SpecimenVariableFields) {
        let thermal_preservation_method = self.thermal_preservation_method.into();

        (
            self.into_common(),
            SpecimenVariableFields {
                type_: SpecimenType::CellPellet,
                embedded_in: None,
                fixative: None,
                thermal_preservation_method: Some(thermal_preservation_method),
            },
        )
    }
}
