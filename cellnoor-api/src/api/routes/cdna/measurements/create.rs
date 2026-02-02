use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    cdna::measurement::{CdnaMeasurement, NewCdnaMeasurement},
};
use cellnoor_schema::{cdna, cdna_measurements};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::util::validate_timestamps,
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_cdna_measurement(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    Json(measurement): Json<NewCdnaMeasurement>,
) -> Result<Json<CdnaMeasurement>, db::Error> {
    let prepared_at = cdna_prepared_at(id, &mut db_conn).await?;

    validate_timestamps(
        (prepared_at, "cdna_prepared_at"),
        (measurement.measured_at(), "measurement_made_at"),
    )?;

    insert_cdna_measurement(id, measurement, &mut db_conn)
        .await
        .map(Json)
}

async fn cdna_prepared_at(
    cdna_id: Uuid,
    db_conn: &mut DbConnection,
) -> Result<Timestamp, db::Error> {
    Ok(cdna::table
        .select(cdna::prepared_at)
        .find(cdna_id)
        .first(db_conn)
        .await
        .map(jiff_diesel::Timestamp::to_jiff)?)
}

pub async fn insert_cdna_measurement(
    cdna_id: Uuid,
    measurement: NewCdnaMeasurement,
    db_conn: &mut DbConnection,
) -> Result<CdnaMeasurement, db::Error> {
    Ok(diesel::insert_into(cdna_measurements::table)
        .values((cdna_measurements::cdna_id.eq(cdna_id), measurement))
        .returning(CdnaMeasurement::as_returning())
        .get_result(db_conn)
        .await?)
}
