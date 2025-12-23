use crate::{
    suspension::measurement::common::{Cells, Nuclei},
    suspension_pool::measurement::common::SuspensionPoolMeasurementFields,
};

pub type CellSuspensionPoolMeasurementCreation = SuspensionPoolMeasurementFields<Cells>;

pub type NucleusSuspensionPoolMeasurementCreation = SuspensionPoolMeasurementFields<Nuclei>;
