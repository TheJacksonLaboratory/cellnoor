use scamplers_models::suspension::{CellSuspensionCreation, NucleusSuspensionCreation};

use crate::validate::Validate;

pub mod measurement;

impl Validate for CellSuspensionCreation {}

impl Validate for NucleusSuspensionCreation {}
