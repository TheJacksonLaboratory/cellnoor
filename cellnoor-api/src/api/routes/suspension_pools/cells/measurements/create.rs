use axum::{extract::State, http::StatusCode};
use cellnoor_models::suspension_pool::{
    SuspensionPoolIdMeasurements,
    measurement::{CellSuspensionPoolMeasurementCreation, SuspensionPoolMeasurement},
};
use diesel::prelude::*;

use crate::{
    api::{
        extract::{ValidPathJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn create_cell_suspension_pool_measurement(
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidPathJson(pool_id, measurement): ValidPathJson<
        SuspensionPoolIdMeasurements,
        CellSuspensionPoolMeasurementCreation,
    >,
) -> ApiResponse<SuspensionPoolMeasurement> {
    let item = inner_handler(state, user, (pool_id, measurement)).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<SuspensionPoolMeasurement>
    for (
        SuspensionPoolIdMeasurements,
        CellSuspensionPoolMeasurementCreation,
    )
{
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<SuspensionPoolMeasurement, db::Error> {
        use cellnoor_schema::suspension_pool_measurements::dsl::*;

        let (p_id, measurement_data) = self;

        Ok(diesel::insert_into(suspension_pool_measurements)
            .values((pool_id.eq(p_id), measurement_data))
            .returning(SuspensionPoolMeasurement::as_returning())
            .get_result(db_conn)?)
    }
}
