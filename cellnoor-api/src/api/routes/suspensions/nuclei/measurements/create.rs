use axum::extract::State;
use cellnoor_models::suspension::{
    SuspensionIdMeasurements,
    measurement::{NucleusSuspensionMeasurementCreation, SuspensionMeasurement},
};
use diesel::prelude::*;
use reqwest::StatusCode;

use crate::{
    api::{
        extract::{ValidPathJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn create_nucleus_suspension_measurement(
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidPathJson(suspension_id, measurement): ValidPathJson<
        SuspensionIdMeasurements,
        NucleusSuspensionMeasurementCreation,
    >,
) -> ApiResponse<SuspensionMeasurement> {
    let item = inner_handler(state, user, (suspension_id, measurement)).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<SuspensionMeasurement>
    for (
        SuspensionIdMeasurements,
        NucleusSuspensionMeasurementCreation,
    )
{
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<SuspensionMeasurement, db::Error> {
        use cellnoor_schema::suspension_measurements::dsl::*;

        let (susp_id, measurement_data) = self;

        Ok(diesel::insert_into(suspension_measurements)
            .values((suspension_id.eq(susp_id), measurement_data))
            .returning(SuspensionMeasurement::as_returning())
            .get_result(db_conn)?)
    }
}
