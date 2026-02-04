use axum::{
    Extension, Json,
    extract::{Path, State},
    http::status::StatusCode,
};
use cellnoor_models::{
    IdParameter,
    library::measurement::{LibraryMeasurement, NewLibraryMeasurement},
};
use cellnoor_schema::{libraries, library_measurements};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::{
        auth::AuthUser, routes::cdna::validate_electrophoretic_measurement,
        util::validate_timestamps,
    },
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_library_measurement(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    Json(measurement): Json<NewLibraryMeasurement>,
) -> Result<Json<LibraryMeasurement>, db::Error> {
    validate_electrophoretic_measurement(measurement.data())?;

    let library_prepared_at = library_prepared_at(id, &mut db_conn).await?;
    validate_timestamps(
        (library_prepared_at, "library_prepared_at"),
        (measurement.measured_at(), "measurement_made_at"),
    )?;

    insert_library_measurement(id, measurement, &mut db_conn)
        .await
        .map(Json)
}

pub async fn insert_library_measurement(
    library_id: Uuid,
    measurement: NewLibraryMeasurement,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<LibraryMeasurement, db::Error> {
    Ok(diesel::insert_into(library_measurements::table)
        .values((library_measurements::library_id.eq(library_id), measurement))
        .returning(LibraryMeasurement::as_returning())
        .get_result(&mut db_conn)
        .await?)
}

async fn library_prepared_at(
    library_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Timestamp, db::Error> {
    Ok(libraries::table
        .select(libraries::prepared_at)
        .find(library_id)
        .first(&mut db_conn)
        .await
        .map(jiff_diesel::Timestamp::to_jiff)?)
}
