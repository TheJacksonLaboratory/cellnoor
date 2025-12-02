use axum::extract::State;
use diesel::prelude::*;
use reqwest::StatusCode;
use scamplers_models::suspension::{
    SuspensionIdMeasurements,
    measurement::{NucleusSuspensionMeasurementCreation, SuspensionMeasurement},
};

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn create_nucleus_suspension_measurement(
    suspension_id: SuspensionIdMeasurements,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<NucleusSuspensionMeasurementCreation>,
) -> ApiResponse<SuspensionMeasurement> {
    let item = inner_handler(state, user, (suspension_id, request)).await?;
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
        use scamplers_schema::suspension_measurements::dsl::*;

        let (susp_id, NucleusSuspensionMeasurementCreation(measurement_data)) = self;

        Ok(diesel::insert_into(suspension_measurements)
            .values((suspension_id.eq(susp_id), measurement_data))
            .returning(SuspensionMeasurement::as_returning())
            .get_result(db_conn)?)
    }
}
