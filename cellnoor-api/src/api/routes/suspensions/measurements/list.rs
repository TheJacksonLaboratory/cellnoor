use axum::{extract::State, http::StatusCode};
use cellnoor_models::suspension::{SuspensionIdMeasurements, measurement::SuspensionMeasurement};
use diesel::prelude::*;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db::{self},
    state::AppState,
};

pub async fn list_measurements(
    suspension_id: SuspensionIdMeasurements,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Vec<SuspensionMeasurement>> {
    let items = inner_handler(state, user, suspension_id).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<SuspensionMeasurement>> for SuspensionIdMeasurements {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<SuspensionMeasurement>, db::Error> {
        use cellnoor_schema::suspension_measurements::dsl::*;

        Ok(SuspensionMeasurement::query()
            .filter(suspension_id.eq(self))
            .load(db_conn)?)
    }
}
