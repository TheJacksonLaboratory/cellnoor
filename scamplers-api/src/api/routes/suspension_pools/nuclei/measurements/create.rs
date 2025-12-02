use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use scamplers_models::suspension_pool::{
    SuspensionPoolIdMeasurements,
    measurement::{NucleusSuspensionPoolMeasurementCreation, SuspensionPoolMeasurement},
};

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn create_nucleus_suspension_pool_measurement(
    pool_id: SuspensionPoolIdMeasurements,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<NucleusSuspensionPoolMeasurementCreation>,
) -> ApiResponse<SuspensionPoolMeasurement> {
    let item = inner_handler(state, user, (pool_id, request)).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<SuspensionPoolMeasurement>
    for (
        SuspensionPoolIdMeasurements,
        NucleusSuspensionPoolMeasurementCreation,
    )
{
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<SuspensionPoolMeasurement, db::Error> {
        use scamplers_schema::suspension_pool_measurements::dsl::*;

        let (p_id, NucleusSuspensionPoolMeasurementCreation(measurement_data)) = self;

        Ok(diesel::insert_into(suspension_pool_measurements)
            .values((pool_id.eq(p_id), measurement_data))
            .returning(SuspensionPoolMeasurement::as_returning())
            .get_result(db_conn)?)
    }
}
