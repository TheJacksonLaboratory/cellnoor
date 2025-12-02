use macro_attributes::base_model;

use crate::{
    suspension::measurement::common::{Cells, Nuclei},
    suspension_pool::measurement::common::SuspensionPoolMeasurementFields,
};

#[base_model]
#[derive(serde::Deserialize)]
pub struct CellSuspensionPoolMeasurementCreation(pub SuspensionPoolMeasurementFields<Cells>);

#[base_model]
#[derive(serde::Deserialize)]
pub struct NucleusSuspensionPoolMeasurementCreation(pub SuspensionPoolMeasurementFields<Nuclei>);
