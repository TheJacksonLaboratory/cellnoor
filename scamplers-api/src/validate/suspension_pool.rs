use diesel::prelude::*;
use scamplers_models::suspension_pool::{SuspensionPoolCreation, SuspensionTagging};
use scamplers_schema::suspensions;

use crate::validate::Validate;

pub mod measurement;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "SuspensionValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("differing suspension contents")]
    SuspensionContent(Vec<String>),
}

impl Validate for SuspensionPoolCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        let suspension_ids = self
            .suspensions
            .as_ref()
            .iter()
            .map(SuspensionTagging::suspension_id);

        let contents: Vec<String> = suspensions::table
            .select(suspensions::content)
            .filter(suspensions::id.eq_any(suspension_ids))
            .distinct()
            .load(db_conn)?;

        if !contents.iter().map(Some).all(|c| c == contents.first()) {
            return Err(Error::SuspensionContent(contents))?;
        }

        Ok(())
    }
}
