use axum::extract::State;
use cellnoor_models::library::{LibraryIdMeasurements, measurement::LibraryMeasurement};
use cellnoor_schema::library_measurements;
use diesel::prelude::*;
use reqwest::StatusCode;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db::{self},
    state::AppState,
};

pub async fn list_measurements(
    library_id: LibraryIdMeasurements,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Vec<LibraryMeasurement>> {
    let item = inner_handler(state, user, library_id).await?;
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
