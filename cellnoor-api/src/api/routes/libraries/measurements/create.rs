use axum::{extract::State, http::StatusCode};
use cellnoor_models::library::{
    LibraryIdMeasurements,
    measurement::{LibraryMeasurement, LibraryMeasurementCreation},
};
use cellnoor_schema::library_measurements;
use diesel::{RunQueryDsl, prelude::*};

use crate::{
    api::{
        extract::{ValidPathJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn create_measurement(
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidPathJson(library_id, measurement): ValidPathJson<
        LibraryIdMeasurements,
        LibraryMeasurementCreation,
    >,
) -> ApiResponse<LibraryMeasurement> {
    let item = inner_handler(state, user, (library_id, measurement)).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<LibraryMeasurement> for (LibraryIdMeasurements, LibraryMeasurementCreation) {
    fn execute(self, db_conn: &mut PgConnection) -> Result<LibraryMeasurement, db::Error> {
        let (cdna_id, measurement) = self;

        Ok(diesel::insert_into(library_measurements::table)
            .values((library_measurements::library_id.eq(cdna_id), measurement))
            .returning(LibraryMeasurement::as_returning())
            .get_result(db_conn)?)
    }
}
