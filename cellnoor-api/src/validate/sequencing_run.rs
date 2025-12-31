use cellnoor_models::sequencing_run::SequencingRunCreation;

use crate::validate::Validate;

mod libraries;

// The database will ensure that `sequencing_run.finished_at` >
// `sequencing_run.begun_at`
impl Validate for SequencingRunCreation {}
