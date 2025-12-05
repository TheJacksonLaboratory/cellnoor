use diesel::prelude::*;
use scamplers_models::suspension_pool::{SuspensionPoolCreation, SuspensionTagging};
use scamplers_schema::suspensions;
use uuid::Uuid;

use crate::{db, validate::Validate};

pub mod measurement;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "SuspensionValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("differing suspension contents")]
    SuspensionContent,
}

impl Validate for SuspensionPoolCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        validate_all_suspensions_have_same_contents(self.suspensions.as_ref(), db_conn)?;

        Ok(())
    }
}

fn validate_all_suspensions_have_same_contents(
    suspension_tagging: &[SuspensionTagging],
    db_conn: &mut PgConnection,
) -> Result<(), super::Error> {
    let suspension_ids = suspension_tagging
        .iter()
        .map(SuspensionTagging::suspension_id);

    let n_distinct_suspension_contents =
        fetch_n_distinct_suspension_contents(suspension_ids, db_conn)?;

    if n_distinct_suspension_contents != 1 {
        Err(Error::SuspensionContent)?;
    }

    Ok(())
}

fn fetch_n_distinct_suspension_contents(
    suspension_ids: impl Iterator<Item = Uuid>,
    db_conn: &mut PgConnection,
) -> Result<i64, db::Error> {
    Ok(suspensions::table
        .filter(suspensions::id.eq_any(suspension_ids))
        .distinct()
        .count()
        .get_result(db_conn)?)
}
