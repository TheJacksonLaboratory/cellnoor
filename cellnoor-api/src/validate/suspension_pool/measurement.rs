use cellnoor_models::suspension_pool::{
    SuspensionPoolId, SuspensionPoolIdMeasurements, measurement::SuspensionPoolMeasurementFields,
};
use diesel::PgConnection;
use jiff::Timestamp;

use crate::{
    db::Operation,
    validate::{Validate, common::validate_timestamps},
};

impl<C> Validate
    for (
        SuspensionPoolIdMeasurements,
        SuspensionPoolMeasurementFields<C>,
    )
{
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        let (SuspensionPoolIdMeasurements(pool_id), measurement) = self;
        validate_suspension_pool_created_before_measurement(
            *pool_id,
            measurement.measured_at(),
            db_conn,
        )?;

        Ok(())
    }
}

fn validate_suspension_pool_created_before_measurement(
    pool_id: impl Into<SuspensionPoolId>,
    measured_at: Timestamp,
    db_conn: &mut PgConnection,
) -> Result<(), crate::validate::Error> {
    let pooled_at = pool_id.into().execute(db_conn)?.pooled_at();
    validate_timestamps(pooled_at, measured_at, "measured_at")?;

    Ok(())
}
