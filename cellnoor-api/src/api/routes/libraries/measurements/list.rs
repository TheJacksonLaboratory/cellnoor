use axum::{extract::State, http::StatusCode};
use cellnoor_models::library::{LibraryIdMeasurements, measurement::LibraryMeasurement};
use cellnoor_schema::library_measurements;
use diesel::prelude::*;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, handle_request},
    },
    db::{self},
    state::AppState,
};

pub async fn list_measurements(
    library_id: LibraryIdMeasurements,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Vec<LibraryMeasurement>> {
    let item = handle_request(state, user, library_id).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Vec<LibraryMeasurement>> for LibraryIdMeasurements {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<LibraryMeasurement>, db::Error> {
        Ok(LibraryMeasurement::query()
            .order_by(library_measurements::measured_at)
            .filter(library_measurements::library_id.eq(self))
            .load(db_conn)?)
    }
}
