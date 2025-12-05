use scamplers_models::suspension::measurement::{
    CellSuspensionMeasurementCreation, NucleusSuspensionMeasurementCreation,
};

use crate::validate::Validate;

impl Validate for CellSuspensionMeasurementCreation {}

impl Validate for NucleusSuspensionMeasurementCreation {}
