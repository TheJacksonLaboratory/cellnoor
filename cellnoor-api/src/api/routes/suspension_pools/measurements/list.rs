use axum::{extract::State, http::StatusCode};
use cellnoor_models::suspension_pool::{
    SuspensionPoolIdMeasurements, measurement::SuspensionPoolMeasurement,
};
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
    pool_id: SuspensionPoolIdMeasurements,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Vec<SuspensionPoolMeasurement>> {
    let items = inner_handler(state, user, pool_id).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<SuspensionPoolMeasurement>> for SuspensionPoolIdMeasurements {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<SuspensionPoolMeasurement>, db::Error> {
        use cellnoor_schema::suspension_pool_measurements::dsl::*;

        Ok(SuspensionPoolMeasurement::query()
            .filter(pool_id.eq(self))
            .load(db_conn)?)
    }
}
