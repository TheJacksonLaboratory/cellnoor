use macro_attributes::base_model;

use crate::suspension::measurement::common::{Cells, Nuclei, SuspensionMeasurementFields};

#[base_model]
#[derive(serde::Deserialize)]
pub struct CellSuspensionMeasurementCreation(pub SuspensionMeasurementFields<Cells>);

#[base_model]
#[derive(serde::Deserialize)]
pub struct NucleusSuspensionMeasurementCreation(pub SuspensionMeasurementFields<Nuclei>);
