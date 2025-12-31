use axum::extract::State;
use cellnoor_models::specimen::{SpecimenIdMeasurements, measurement::SpecimenMeasurement};
use cellnoor_schema::specimen_measurements;
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
    specimen_id: SpecimenIdMeasurements,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Vec<SpecimenMeasurement>> {
    let item = inner_handler(state, user, specimen_id).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Vec<SpecimenMeasurement>> for SpecimenIdMeasurements {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<SpecimenMeasurement>, db::Error> {
        Ok(SpecimenMeasurement::query()
            .order_by(specimen_measurements::measured_at)
            .filter(specimen_measurements::specimen_id.eq(self))
            .load(db_conn)?)
    }
}
