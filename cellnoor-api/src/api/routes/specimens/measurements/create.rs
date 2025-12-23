use axum::{extract::State, http::StatusCode};
use cellnoor_models::specimen::{
    SpecimenIdMeasurements,
    measurement::{SpecimenMeasurement, SpecimenMeasurementCreation},
};
use cellnoor_schema::specimen_measurements;
use diesel::{RunQueryDsl, prelude::*};

use crate::{
    api::{
        extract::{ValidPathJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn create_measurement(
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidPathJson(specimen_id, measurement): ValidPathJson<
        SpecimenIdMeasurements,
        SpecimenMeasurementCreation,
    >,
) -> ApiResponse<SpecimenMeasurement> {
    let item = inner_handler(state, user, (specimen_id, measurement)).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<SpecimenMeasurement> for (SpecimenIdMeasurements, SpecimenMeasurementCreation) {
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
