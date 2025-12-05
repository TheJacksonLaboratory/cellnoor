use axum::{extract::State, http::StatusCode};
use diesel::{RunQueryDsl, prelude::*};
use scamplers_models::library::{
    LibraryIdMeasurements,
    measurement::{LibraryMeasurement, LibraryMeasurementCreation},
};
use scamplers_schema::library_measurements;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn create_measurement(
    library_id: LibraryIdMeasurements,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<LibraryMeasurementCreation>,
) -> ApiResponse<LibraryMeasurement> {
    let item = inner_handler(state, user, (library_id, request)).await?;
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
