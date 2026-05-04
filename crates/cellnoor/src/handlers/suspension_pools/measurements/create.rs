use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    suspension::SuspensionContent,
    suspension_pool::measurement::{
        NewSuspensionPoolMeasurement, SuspensionPoolMeasurement, SuspensionPoolMeasurementData,
    },
};
use cellnoor_schema::{
    suspension_pool_measurements, suspension_pooling, suspension_pools, suspensions,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::util::validate_timestamps,
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_suspension_pool_measurement(
    _: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    Json(measurement): Json<NewSuspensionPoolMeasurement>,
) -> Result<Json<SuspensionPoolMeasurement>, db::Error> {
    let (pooled_at, suspension_content) = suspension_info(id, &db_conn).await?;

    validate_timestamps(
        (pooled_at, "suspension_pool_pooled_at"),
        (measurement.measured_at(), "measurement_made_at"),
    )?;
    validate_measurement_content_matches_suspension_content(
        suspension_content,
        measurement.data(),
    )?;

    insert_suspension_pool_measurement(id, measurement, &db_conn)
        .await
        .map(Json)
}

fn validate_measurement_content_matches_suspension_content(
    suspension_content: SuspensionContent,
    measurement: &SuspensionPoolMeasurementData,
) -> Result<(), db::DataError> {
    // Using a match statement is better than a let guard because if you ever add
    // another measurement variant, the compiler protects you
    let measurement_content = match measurement {
        SuspensionPoolMeasurementData::Concentration(inner) => inner.numerator_unit(),
        SuspensionPoolMeasurementData::MeanDiameter(inner) => inner.object(),
        SuspensionPoolMeasurementData::Viability(_) | SuspensionPoolMeasurementData::Volume(_) => {
            return Ok(());
        }
    };

    if suspension_content != measurement_content {
        return Err(db::DataError::Other {
            message: "measurement content must match suspension content".to_owned(),
        });
    }

    Ok(())
}

async fn suspension_info(
    suspension_pool_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(Timestamp, SuspensionContent), db::Error> {
    Ok(suspension_pools::table
        .inner_join(suspension_pooling::table.inner_join(suspensions::table))
        .select((suspension_pools::pooled_at, suspensions::content))
        .filter(suspension_pools::id.eq(suspension_pool_id))
        .first(&mut db_conn)
        .await
        .map(|(t, c): (jiff_diesel::Timestamp, _)| (t.to_jiff(), c))?)
}

pub async fn insert_suspension_pool_measurement(
    suspension_pool_id: Uuid,
    measurement: NewSuspensionPoolMeasurement,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<SuspensionPoolMeasurement, db::Error> {
    Ok(diesel::insert_into(suspension_pool_measurements::table)
        .values((
            suspension_pool_measurements::pool_id.eq(suspension_pool_id),
            measurement,
        ))
        .returning(SuspensionPoolMeasurement::as_returning())
        .get_result(&mut db_conn)
        .await?)
}
