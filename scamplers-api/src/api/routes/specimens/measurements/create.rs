use axum::{extract::State, http::StatusCode};
use diesel::{RunQueryDsl, prelude::*};
use scamplers_models::specimen::{SpecimenIdMeasurements, measurement::SpecimenMeasurement};
use scamplers_schema::specimen_measurements;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn create_measurement(
    specimen_id: SpecimenIdMeasurements,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<SpecimenMeasurement>,
) -> ApiResponse<SpecimenMeasurement> {
    let item = inner_handler(state, user, (specimen_id, request)).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<SpecimenMeasurement> for (SpecimenIdMeasurements, SpecimenMeasurement) {
    fn execute(self, db_conn: &mut PgConnection) -> Result<SpecimenMeasurement, db::Error> {
        let (specimen_id, measurement) = self;

        Ok(diesel::insert_into(specimen_measurements::table)
            .values((
                specimen_measurements::specimen_id.eq(specimen_id),
                measurement,
            ))
            .returning(SpecimenMeasurement::as_returning())
            .get_result(db_conn)?)
    }
}
