use macro_attributes::{base_model, insert};
#[cfg(feature = "app")]
use scamplers_schema::suspension_measurements;

use crate::suspension::{
    Suspension,
    measurements::common::{Cells, Nuclei, SuspensionMeasurementFields},
};

#[base_model]
#[derive(serde::Deserialize)]
pub struct CellSuspensionMeasurementCreation(pub SuspensionMeasurementFields<Cells>);

#[base_model]
#[derive(serde::Deserialize)]
pub struct NucleusSuspensionMeasurementCreation(pub SuspensionMeasurementFields<Nuclei>);
