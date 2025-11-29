use axum::extract::State;
use diesel::prelude::*;
use reqwest::StatusCode;
use scamplers_models::suspension::measurement::{
    CellSuspensionMeasurementCreation, NucleusSuspensionMeasurementCreation, SuspensionMeasurement,
};
use scamplers_schema::suspension_measurements::dsl::*;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler, suspensions::measurements::MeasurementsEndpoint},
    },
    db,
    state::AppState,
};

pub async fn create_nucleus_suspension_measurement(
    _: MeasurementsEndpoint,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<NucleusSuspensionMeasurementCreation>,
) -> ApiResponse<SuspensionMeasurement> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<SuspensionMeasurement> for NucleusSuspensionMeasurementCreation {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<SuspensionMeasurement, db::Error> {
        Ok(diesel::insert_into(suspension_measurements)
            .values(self.0)
            .returning(SuspensionMeasurement::as_returning())
            .get_result(db_conn)?)
    }
}
