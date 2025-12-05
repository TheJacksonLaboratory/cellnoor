use scamplers_models::suspension_pool::measurement::{
    CellSuspensionPoolMeasurementCreation, NucleusSuspensionPoolMeasurementCreation,
};

use crate::validate::Validate;

impl Validate for CellSuspensionPoolMeasurementCreation {}
impl Validate for NucleusSuspensionPoolMeasurementCreation {}
