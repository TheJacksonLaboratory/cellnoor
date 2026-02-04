use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    suspension::{
        SuspensionContent,
        measurement::{NewSuspensionMeasurement, SuspensionMeasurement, SuspensionMeasurementData},
    },
};
use cellnoor_schema::{suspension_measurements, suspensions};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use jiff_diesel::NullableTimestamp;
use uuid::Uuid;

use crate::{
    api::util::validate_timestamps,
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_suspension_measurement(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    Json(measurement): Json<NewSuspensionMeasurement>,
) -> Result<Json<SuspensionMeasurement>, db::Error> {
    let (suspension_content, suspension_created_at) = suspension_info(id, &mut db_conn).await?;

    validate_measurement_content_matches_suspension_content(
        suspension_content,
        measurement.data(),
    )?;

    if let Some(suspension_created_at) = suspension_created_at {
        validate_timestamps(
            (suspension_created_at, "suspension_created_at"),
            (measurement.measured_at(), "measurement_made_at"),
        )?;
    }

    insert_suspension_measurement(id, measurement, &mut db_conn)
        .await
        .map(Json)
}

fn validate_measurement_content_matches_suspension_content(
    suspension_content: SuspensionContent,
    measurement: &SuspensionMeasurementData,
) -> Result<(), db::DataError> {
    // Using a match statement is better than a let guard because if you ever add
    // another measurement variant, the compiler protects you
    let measurement_content = match measurement {
        SuspensionMeasurementData::Concentration {
            numerator_unit: measurement_content,
            ..
        }
        | SuspensionMeasurementData::MeanDiameter {
            object: measurement_content,
            ..
        } => *measurement_content,
        SuspensionMeasurementData::Viability { .. } | SuspensionMeasurementData::Volume { .. } => {
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
    suspension_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(SuspensionContent, Option<jiff::Timestamp>), db::Error> {
    fn map_result(
        (content, created_at): (SuspensionContent, NullableTimestamp),
    ) -> (SuspensionContent, Option<Timestamp>) {
        (content, created_at.to_jiff())
    }

    Ok(suspensions::table
        .select((suspensions::content, suspensions::created_at))
        .find(suspension_id)
        .first(&mut db_conn)
        .await
        .map(map_result)?)
}

async fn insert_suspension_measurement(
    suspension_id: Uuid,
    measurement: NewSuspensionMeasurement,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<SuspensionMeasurement, db::Error> {
    Ok(diesel::insert_into(suspension_measurements::table)
        .values((
            suspension_measurements::suspension_id.eq(suspension_id),
            measurement,
        ))
        .returning(SuspensionMeasurement::as_returning())
        .get_result(&mut db_conn)
        .await?)
}
