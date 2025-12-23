use cellnoor_models::{
    suspension::SuspensionContent,
    suspension_pool::{SuspensionPoolCreation, SuspensionTagging},
};
use cellnoor_schema::{specimens, suspensions};
use diesel::prelude::*;
use jiff::Timestamp;
use uuid::Uuid;

use crate::validate::{
    Validate, chromium_run::join_suspensions_to_specimens, common::validate_timestamps,
};

pub mod measurement;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "SuspensionPoolValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("differing suspension contents")]
    SuspensionContent,
}

impl Validate for SuspensionPoolCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        let suspension_ids: Vec<_> = self
            .suspensions
            .as_ref()
            .iter()
            .map(SuspensionTagging::suspension_id)
            .collect();

        let suspension_data = fetch_suspensions_data(&suspension_ids, db_conn)?;

        validate_all_suspensions_have_same_contents(&suspension_data)?;
        validate_suspensions_created_before_pooled(&suspension_data, self.inner.pooled_at())?;

        Ok(())
    }
}

fn validate_all_suspensions_have_same_contents(
    suspension_data: &[(Timestamp, SuspensionContent)],
) -> Result<(), super::Error> {
    let Some(first) = suspension_data.first() else {
        return Ok(());
    };

    if !suspension_data
        .iter()
        .all(|(_, content)| first.1 == *content)
    {
        Err(Error::SuspensionContent)?;
    }

    Ok(())
}

fn validate_suspensions_created_before_pooled(
    suspension_data: &[(Timestamp, SuspensionContent)],
    pooled_at: Timestamp,
) -> Result<(), super::Error> {
    for (ts, _) in suspension_data {
        validate_timestamps(*ts, pooled_at, "pooled_at")?;
    }

    Ok(())
}

fn fetch_suspensions_data(
    suspension_ids: &[Uuid],
    db_conn: &mut PgConnection,
) -> Result<Vec<(Timestamp, SuspensionContent)>, super::Error> {
    let data: Vec<(jiff_diesel::NullableTimestamp, jiff_diesel::Timestamp, _)> =
        join_suspensions_to_specimens(suspension_ids)
            .select((
                suspensions::created_at,
                specimens::received_at,
                suspensions::content,
            ))
            .load(db_conn)?;

    Ok(data
        .into_iter()
        .map(|(t1, t2, content)| (t1.to_jiff().unwrap_or_else(|| t2.to_jiff()), content))
        .collect())
}
