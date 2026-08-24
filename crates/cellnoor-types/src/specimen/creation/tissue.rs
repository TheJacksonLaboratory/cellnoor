use macro_attributes::base_model;

use crate::specimen::{Fixative, ThermalPreservationMethod};

#[base_model]
#[derive(Copy)]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "snake_case", tag = "preservation_state")
)]
pub enum TissueFields {
    Fixed {
        fixative: Fixative,
    },
    Fresh,
    ThermallyPreserved {
        thermal_preservation_method: ThermalPreservationMethod,
    },
}
